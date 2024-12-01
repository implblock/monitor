use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    collections::HashMap,
    str::FromStr,
};

use tokio::{
    io::{self, AsyncReadExt},
    fs::{read_dir, File},
};

use serde::{Deserialize, Serialize};
use futures_util::TryStreamExt;
use monitor::probe::Probe;
use anyhow::anyhow;

use tokio_stream::wrappers::ReadDirStream;

#[derive(
    PartialEq, PartialOrd,
    Deserialize,
    Serialize,
    Debug,
)]

pub struct Interface {
    pub path: PathBuf,
    pub name: String,
    pub stats: Stats,
    pub info: Info,
}

#[derive(
    PartialEq, PartialOrd,
    Deserialize,
    Serialize,
    Debug,
)]

pub struct Stats {
    pub tx_packets: usize,
    pub rx_packets: usize,
    pub rx_bytes: usize,
    pub tx_bytes: usize,
}

#[derive(
    PartialEq, PartialOrd,
    Deserialize,
    Serialize,
    Debug,
)]

pub enum OperState {
    Unknown,
    Down,
    Up,
}

impl FromStr for OperState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unknown" => Ok(Self::Unknown),
            "down" => Ok(Self::Down),
            "up" => Ok(Self::Up),
            s => Err(anyhow!(
                "invalid OperState '{}'",
                s,
            )),
        }
    }
}

impl Display for OperState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Down => write!(f, "down"),
            Self::Up => write!(f, "up"),
        }
    }
}

#[derive(
    PartialEq, PartialOrd,
    Deserialize,
    Serialize,
    Debug,
)]

pub struct Info {
    pub operstate: OperState,
    pub address: String,
    pub speed: usize,
}

#[derive(
    Deserialize,
    PartialEq,
    Serialize,
    Default,
    Debug,
)]

pub struct Network {
    pub interfaces: Vec<Interface>,
    // Possibly other stuff
}

#[derive(
    thiserror::Error,
    Debug,
)]

pub enum Error {
    #[error("io error occurred while getting network information: {0}")]
    Io(#[from] io::Error),
    #[error("info '{info}' not found for interface '{interface}'")]
    MissingInfo {
        interface: String,
        info: String,
    },
    #[error("stat '{stat}' not found for interface '{interface}'")]
    MissingStat {
        interface: String,
        stat: String,
    },
    #[error("interface '{interface}' is in an invalidstate")]
    InvalidLinkState {
        interface: String,
    },
    #[error("failed to parse stat '{stat}' {source}'")]
    ParseError {
        source: anyhow::Error,
        stat: String,
    },
}

impl Probe for Network {
    type Output = Self;

    type Error = Error;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let nw_dir = std::env::var("NET")
            .unwrap_or("/sys/class/net/".into());

        let mut rdir = read_dir(nw_dir).await?;

        let mut interfaces = Vec::new();

        while let Some(ent) = rdir.next_entry().await? {
            if ent.file_type().await?.is_file() {
                continue;
            }

            let name = ent.file_name()
                .to_string_lossy()
                .to_string();

            let path = ent.path();

            let stats = get_stats(
                &path
            ).await?;

            let info = get_info(
                &path
            ).await?;

            let interface = Interface {
                stats,
                name,
                path,
                info,
            };

            interfaces.push(interface);
        }

        Ok(Self {
            interfaces,
        })
    }
}


async fn get_stats(interface: &Path) -> Result<Stats, Error> {
    let path = format!("{}/statistics", interface.to_string_lossy());

    let rstats = ReadDirStream::new(read_dir(path).await?)
        .try_filter_map(|x| async move {
            if !x.file_type().await?.is_file() {
                return Ok(None::<(String, PathBuf)>);
            }

            let name = x.file_name().to_string_lossy()
                .to_string();

            let path = x.path();

            Ok(Some((name, path)))
        })
        .try_collect::<HashMap<String, PathBuf>>().await?;

    // TODO
    // add more stats

    let [
        rx_packets,
        tx_packets,
        rx_bytes,
        tx_bytes,
    ] = [
        "rx_packets",
        "tx_packets",
        "rx_bytes",
        "tx_bytes",
    ].map(|x| async {
        match rstats.get(x) {
            None => Err(Error::MissingStat {
                interface: interface.to_string_lossy()
                    .to_string(),
                    stat: x.to_string(),
            }),
            Some(x) => read_parse(x).await,
        }
    });

    Ok(Stats {
        rx_packets: rx_packets.await?,
        tx_packets: tx_packets.await?,
        rx_bytes: rx_bytes.await?,
        tx_bytes: tx_bytes.await?,
    })
}

async fn get_info(interface: &PathBuf) -> Result<Info, Error> {
    let rinfo = ReadDirStream::new(read_dir(interface).await?)
        .try_filter_map(|x| async move {
            if !x.file_type().await?.is_file() {
                return Ok(None::<(String, PathBuf)>)
            }

            let name = x.file_name()
                .to_string_lossy()
                .to_string();

            let path = x.path();

            Ok(Some((name, path)))
        })
        .try_collect::<HashMap<String, PathBuf>>().await?;

    let operstate: OperState = match rinfo.get("operstate") {
        None => return Err(Error::MissingInfo {
            interface: interface.to_string_lossy()
                .to_string(),
            info: "operstate".to_string(),
        }),
        Some(x) => read_parse(x).await?,
    };

    let speed = match rinfo.get("speed") {
        None => return Err(Error::MissingInfo {
            interface: interface.to_string_lossy()
                .to_string(),
            info: "speed".to_string(),
        }),
        Some(x) => read_parse::<usize>(x).await,
    };

    let speed = match (speed, &operstate) {
        (Err(_), OperState::Up) => return Err(Error::InvalidLinkState {
            interface: interface.to_string_lossy()
                .to_string(),
        }),
        (Err(_), OperState::Unknown | OperState::Down) => 0,
        (Ok(s), _) => s,
    };

    let address = match rinfo.get("address") {
        None => return Err(Error::MissingInfo {
            interface: interface.to_string_lossy()
                .to_string(),
            info: "address".to_string(),
        }),
        Some(x) => read_parse(x).await?,
    };

    Ok(Info {
        operstate,
        address,
        speed,
    })
}

async fn read_parse<T>(
    path: &PathBuf
) -> Result<T, Error>
where
    <T as FromStr>::Err: Debug + Display,
    T: FromStr,
{
    let mut file = File::open(path).await?;
    let mut str = String::new();

    file.read_to_string(&mut str)
        .await?;

    str.trim().parse::<T>()
        .map_err(|x| {
            Error::ParseError {
                source: anyhow!(x.to_string()),
                stat: path.to_string_lossy()
                    .to_string(),
            }
        })
}
