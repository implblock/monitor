use std::{collections::HashMap, num::{IntErrorKind, ParseIntError}};

use monitor_core::probe::Probe;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File, 
    io::{
        self, AsyncBufReadExt, BufReader
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

/// Most of the memory information from
/// the linux /proc/meminfo file
///
/// This file updates on-read, so
/// extremely frequent probing is
/// discouraged.
pub struct Memory {
    pub inactive_anonymous: u64,
    pub active_anonymous: u64,
    pub inactive_file: u64,
    pub active_file: u64,
    pub unevictable: u64,
    pub swap_cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub available: u64,
    pub inactive: u64,
    pub m_locked: u64,
    pub buffers: u64,
    pub cached: u64,
    pub active: u64,
    pub dirty: u64,
    pub total: u64,
    pub free: u64,
}

impl Memory {
    /// Returns the difference of the total
    /// memory and the free memory.
    pub fn used(&self) -> u64 {
        self.total - self.free
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error occurred getting meminfo: {0}")]
    Io(#[from] io::Error),
    #[error("invalid format, expected kB: {0}")]
    InvalidFormat(String),
    #[error("missing field from memory: {0}")]
    MissingField(String),
    #[error("failed to parse memory: {0}")]
    ParseInt(ParseIntError),
    #[error("missing colon at: {0}")]
    MissingColon(String),
    #[error("value is empty: {0}")]
    ValueEmpty(String),
    #[error("key is empty: {0}")]
    KeyEmpty(String),
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

        let mut memmap = HashMap::new();

        while let Some(x) = meminfo.next_line().await? {
            if !x.contains(':') {
                return Err(Error::MissingColon(x));
            }

            let mut parts = x.split(':')
                .map(|x| x.trim().to_owned());

            let k = match parts.next() {
                None => return Err(Error::KeyEmpty(x)),
                Some(x) if x.is_empty() => {
                    return Err(Error::KeyEmpty(x));
                }
                Some(x) => x,
            };

            let v = match parts.next() {
                None => return Err(Error::ValueEmpty(x)),
                Some(x) if x.trim().is_empty() => {
                    return Err(Error::ValueEmpty(x));
                }
                Some(x) => x.trim().to_owned(),
            };

            if !v.trim().contains("kB") {
                return Err(Error::InvalidFormat(x));
            }

            let v = v.replace("kB", "").trim()
                .parse::<u64>().map_err(|e| match e.kind() {
                    IntErrorKind::Empty => Error::ValueEmpty(x),
                    _ => Error::ParseInt(e),
                })?;

            memmap.insert(k, v);
        }

        let [
            inactive_anonymous,
            active_anonymous,
            inactive_file,
            active_file,
            unevictable,
            swap_cached,
            swap_total,
            swap_free,
            available,
            inactive,
            m_locked,
            buffers,
            cached,
            active,
            dirty,
            total,
            free,
        ] = [
            "Inactive(anon)",
            "Active(anon)",
            "Inactive(file)",
            "Active(file)",
            "Unevictable",
            "SwapCached",
            "SwapTotal",
            "SwapFree",
            "MemAvailable",
            "Inactive",
            "Mlocked",
            "Buffers",
            "Cached",
            "Active",
            "Dirty",
            "MemTotal",
            "MemFree",
        ].try_map(|x| {
            memmap.get(x).copied().ok_or(
                Error::MissingField(x.to_string())
            )
        })?;

        Ok(Self {
            inactive_anonymous,
            active_anonymous,
            inactive_file,
            active_file,
            unevictable,
            swap_cached,
            swap_total,
            swap_free,
            available,
            inactive,
            m_locked,
            buffers,
            cached,
            active,
            dirty,
            total,
            free,
        })
    }
}
