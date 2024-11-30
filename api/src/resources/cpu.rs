use std::num::ParseIntError;

use monitor_core::probe::Probe;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use tokio::{
    fs::File,
    io::{
        AsyncBufReadExt,
        BufReader,
        self,
    }
};


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

pub struct Cpu {
    pub usage: Usage,
    // TODO make other
    // stuff here--
}


impl Probe for Cpu {
    type Error = <Usage as Probe>::Error;

    type Output = Cpu;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let usage = Usage::probe().await?;

        Ok(Self {
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
                .map_err(|x| UsageError::ParseInt(x))
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
