use std::{collections::HashMap, num::ParseIntError, path::PathBuf};
use futures_util::{stream, StreamExt, TryStreamExt};
use tokio_stream::wrappers::ReadDirStream;
use serde::{Deserialize, Serialize};
use monitor::probe::Probe;
use thiserror::Error;

use tokio::{
    fs::{read_dir, File},
    io::{
        AsyncBufReadExt,
        AsyncReadExt,
        BufReader,
        self,
    },
};


#[derive(
    PartialOrd, Ord,
    PartialEq, Eq,
    Deserialize,
    Serialize,
    Default,
    Debug,
    Clone,
)]

/// Usage, thermal, diagnostics, and
/// other useful information about
/// the Cpu
///
/// This may use kernel file system
/// callbacks to get information
pub struct Cpu {
    pub cores: Vec<Core>,
    pub usage: Usage,
    // TODO make other
    // stuff here--
}

#[derive(
    Error,
    Debug,
)]

pub enum CpuError {
    #[error("an io error occurred geting cpu info: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    UsageError(#[from] UsageError),
    #[error(transparent)]
    CoreError(#[from] CoreError),
}

#[derive(
    Deserialize,
    PartialOrd,
    Serialize,
    PartialEq,
    Default,
    Debug,
    Clone,
    Copy,
    Ord,
    Eq,
)]

pub struct Core {
    pub max_temp: u64,
    pub count: usize,
    pub temp: u64,
    pub crit: u64,
}

#[derive(
    Error,
    Debug,
)]

pub enum CoreError {
    #[error("an io error occurred getting info about cpu cores: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse integer from core information: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("invalid core label: {0}")]
    InvalidLabel(String),
}

pub struct Cores;

impl Probe for Cores {
    type Output = Vec<Core>;

    type Error = CoreError;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let sysfsct =
            "/sys/devices/platform/coretemp.0/hwmon/hwmon4/";

        let coretemp_dir = std::env::var("CORETEMP")
            .unwrap_or(sysfsct.into());

        let temps = ReadDirStream::new(
            read_dir(coretemp_dir).await?
        ).try_filter_map(|x| async move {
            if !x.file_type().await?.is_file() {
                return Ok(None::<(String, PathBuf)>);
            }

            let fname = x.file_name();

            let fname = fname
                .to_string_lossy();

            if fname.starts_with("temp") {
                let fname = fname
                    .to_string();

                let path = x.path();

                Ok(Some((fname, path)))
            } else {
                Ok(None)
            }
        })
        .try_collect::<HashMap<String, PathBuf>>()
            .await?;

        #[allow(unused)]
        struct Coretemp {
            crit_alarm: u64,
            label: String,
            num: usize,
            input: u64,
            crit: u64,
            max: u64,
        }

        let temps = tokio_stream::StreamExt::map_while(
            stream::iter(1usize..), |x|
        {
            let crit_alarm = temps.get(
                &format!("temp{x}_crit_alarm")
            )?;

            let input = temps.get(
                &format!("temp{x}_input")
            )?;

            let label = temps.get(
                &format!("temp{x}_label")
            )?;

            let crit = temps.get(
                &format!("temp{x}_crit")
            )?;

            let max = temps.get(
                &format!("temp{x}_max")
            )?;

            let a = (
                crit_alarm,
                input,
                label,
                crit,
                max,
                x,
            );

            Some(a)
        })
        // fix this at some point,
        // could make way faster
        .then(|(
            crit_alarm,
            input,
            label,
            crit,
            max,
            num,
        )| async move {

            let mut buf = [0; 1024];

            let n = File::open(crit_alarm).await?
                .read(&mut buf).await?;

            let crit_alarm = String::from_utf8_lossy(
                &buf[..n]
            ).trim().parse::<u64>()?;

            let n = File::open(input).await?
                .read(&mut buf).await?;

            let input = String::from_utf8_lossy(
                &buf[..n]
            ).trim().parse::<u64>()?;

            let n = File::open(label).await?
                .read(&mut buf).await?;

            let label = String::from_utf8_lossy(
                &buf[..n]
            ).to_string();

            let n = File::open(crit).await?
                .read(&mut buf).await?;

            let crit = String::from_utf8_lossy(
                &buf[..n]
            ).trim().parse::<u64>()?;

            let n = File::open(max).await?
                .read(&mut buf).await?;

            let max = String::from_utf8_lossy(
                &buf[..n]
            ).trim().parse::<u64>()?;

            let coretemp = Coretemp {
                crit_alarm,
                input,
                label,
                crit,
                max,
                num,
            };

            Ok::<Coretemp, CoreError>(
                coretemp,
            )
        })
        .try_collect::<Vec<Coretemp>>().await?;

        let cores = temps.iter().filter(|x| {
            x.label.starts_with("Core")
        })
        .map(|x| {
            let num = x.label.trim().split_at_checked(
                5
            ).ok_or(CoreError::InvalidLabel(
                x.label.to_owned()
            ))?.1;

            let count = num.trim()
                .parse::<usize>()?;

            let core = Core {
                max_temp: x.max,
                temp: x.input,
                crit: x.crit,
                count,
            };

            Ok::<Core, CoreError>(core)
        })
        .try_collect()?;

        Ok(cores)
    }
}

impl Probe for Cpu {
    type Error = CpuError;

    type Output = Cpu;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let usage = Usage::probe().await?;
        let cores = Cores::probe().await?;

        Ok(Self {
            cores,
            usage,
        })
    }
}

#[derive(
    PartialOrd, Ord,
    PartialEq, Eq,
    Deserialize,
    Serialize,
    Default,
    Debug,
    Clone,
    Copy,
)]

/// Cpu usage from the /proc/stat
/// file. Mostly information about
/// the processes the cpu is handling
pub struct Usage {
    pub guest_nice: u64,
    pub softirq: u64,
    pub system: u64,
    pub iowait: u64,
    pub steal: u64,
    pub guest: u64,
    pub idle: u64,
    pub user: u64,
    pub nice: u64,
    pub irq: u64,
}

#[derive(
    Error,
    Debug,
)]

pub enum UsageError {
    #[error("stat returned an invalid amount of columns in the cpu line")]
    InvalidCpuLine,
    #[error("io occurred getting cpu info: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse int: {0}")]
    ParseInt(ParseIntError),
    #[error("cpu is missing from stat")]
    CpuMissing,
}

impl Probe for Usage {
    type Output = Usage;

    type Error = UsageError;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let stat_file = std::env::var("STAT")
            .unwrap_or("/proc/stat".into());

        let file = File::open(stat_file).await?;

        let mut cpu = String::new();

        BufReader::new(file).read_line(&mut cpu)
            .await?;

        if !cpu.starts_with("cpu") {
            return Err(UsageError::CpuMissing);
        }

        let cpu = cpu.split_off(3);

        let parts =
            cpu.split_whitespace();

        let [
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
        ]: [
            u64; 10
        ] = parts.map(|x| x.parse::<u64>()
                .map_err(UsageError::ParseInt)
            )
            .try_collect::<Vec<u64>>()?
            .try_into()
            .map_err(|_| UsageError::InvalidCpuLine)?;

        Ok(Self {
            guest_nice,
            softirq,
            system,
            iowait,
            guest,
            steal,
            user,
            idle,
            nice,
            irq,
        })
    }
}
