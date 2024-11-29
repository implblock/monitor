use std::num::ParseIntError;

use monitor_core::probe::Probe;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File, 
    io::{
        self, AsyncBufReadExt, AsyncWriteExt, BufReader
    }
};

#[derive(
    PartialOrd, Ord,
    PartialEq, Eq,
    Deserialize,
    Serialize,
    Default,
    Clone,
    Debug,
    Copy,
)]

// Memory stats in KB
pub struct Memory {
    pub total: u64,
    pub free: u64,
    // TODO maybe add more
}

impl Memory {
    pub fn used(&self) -> u64 {
        self.total - self.free
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error occurred getting meminfo: {0}")]
    Io(#[from] io::Error),
    #[error("memory is invalid: {0}")]
    InvalidMemory(#[from] ParseIntError),
    #[error("total memory not found")]
    TotalNotFound,
    #[error("free memory not found")]
    FreeNotFound,
}

impl Probe for Memory {
    type Output = Memory;

    type Error = Error;

    async fn probe() -> Result<Self::Output, Self::Error> {
        let meminfo_file = std::env::var("MEMINFO")
            .unwrap_or("/proc/meminfo".into());

        let file = File::open(meminfo_file).await?;

        let bufreader = BufReader::new(file);

        let mut meminfo = bufreader.lines();

        let (total, free) = (meminfo.next_line().await?.ok_or(
            Error::TotalNotFound
        ).map(|x| {
            let x = x.replace("kB", "");
            let x = x.replace(' ', "");

            let mut parts = x.split(':');

            if parts.next().is_none_or(|x| x != "MemTotal") {
                Err(Error::TotalNotFound)
            } else {
                parts.next()
                    .ok_or(Error::TotalNotFound)
                    .and_then(|x| Ok(x.parse::<u64>()?))
            }
        }).and_then(|x| x)?,
        meminfo.next_line().await?.ok_or(
            Error::FreeNotFound
        ).map(|x| {
            let x = x.replace("kB", "");
            let x = x.replace(' ', "");

            let mut parts = x.split(':');

            if parts.next().is_none_or(|x| x != "MemFree") {
                Err(Error::FreeNotFound)
            } else {
                parts.next()
                    .ok_or(Error::FreeNotFound)
                    .and_then(|x| Ok(x.parse::<u64>()?))
            }
        }).and_then(|x| x)?);

        let mut inner = meminfo.into_inner();

        inner.flush().await?;

        Ok(Self {
            total,
            free,
        })
    }
}

#[tokio::test]
pub async fn test_probe_mem() -> crate::Any {
    let total = 32697472;
    let free = 22874640;

    let data = format!(
    "MemTotal:       {total} kB\n\
    MemFree:        {free} kB"
    );

    crate::testing::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
       &data, 
    ).await?;

    let mem = Memory::probe().await?;

    assert_eq!(mem.total, total, "invalid mem total");
    assert_eq!(mem.free, free, "invalid mem free");

    Ok(())
}
