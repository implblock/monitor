use std::time::Duration;

use monitor_core::probe::Probe;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{fs::File, io::{self, AsyncBufReadExt, BufReader}};

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

pub struct Uptime {
    pub uptime: Duration,
    pub idle: Duration,
}

#[derive(
    Error,
    Debug,
)]

pub enum Error {
    #[error("io error occurred getting uptime: {0}")]
    Io(#[from] io::Error),
    #[error("the uptime received is invalid")]
    InvalidUptime,
    #[error("uptime file is empty")]
    Empty,
}

impl Probe for Uptime {
    type Output = Uptime;

    type Error = Error;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let uptime_file = std::env::var("UPTIME")
            .unwrap_or("/proc/uptime".into());

        let file = File::open(uptime_file).await?;

        let mut uptime = String::new();

        BufReader::new(file).read_line(&mut uptime)
            .await?;

        if uptime.is_empty() {
            return Err(Error::Empty);
        }

        let parts =
            uptime.split_whitespace();

        let [
            uptime,
            idle,
        ]: [
            Duration; 2
        ] = parts.flat_map(|x| x.parse::<f32>())
            .map(|x| Duration::from_secs_f32(x))
            .collect::<Vec<Duration>>()
            .try_into()
            .map_err(|_| Error::InvalidUptime)?;

        Ok(Self {
            uptime,
            idle,
        })
    }
}

#[tokio::test]
pub async fn test_probe_uptime() -> crate::Any {
    let data = "100.0 10.0";

    crate::testing::point_env_file(
        "UPTIME",
        "/tmp/uptime",
        data,
    ).await?;

    let uptime =
        Uptime::probe().await?;

    assert_eq!(
        uptime.uptime,
        Duration::from_secs_f32(
            100.0
        ),
        "invalid uptime"
    );
    assert_eq!(
        uptime.idle,
        Duration::from_secs_f32(
            10.0,
        ),
        "invalid idle time",
    );

    Ok(())
}
