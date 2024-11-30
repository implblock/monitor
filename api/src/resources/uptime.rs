use std::{num::ParseFloatError, time::Duration};

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
    #[error("the uptime received has an invalid amount of columns")]
    InvalidUptime,
    #[error("io error occurred getting uptime: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse int: {0}")]
    ParseFloat(ParseFloatError),
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
        ] = parts.map(|x| x.parse::<f32>()
                .map(Duration::from_secs_f32)
                .map_err(Error::ParseFloat))
            .try_collect::<Vec<Duration>>()?
            .try_into()
            .map_err(|_| Error::InvalidUptime)?;

        Ok(Self {
            uptime,
            idle,
        })
    }
}
