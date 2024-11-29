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
    usage: Usage,
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
    #[error("stat returned an invalid or out of date cpu line")]
    InvalidCpuLine,
    #[error("io occurred getting cpu info: {0}")]
    Io(#[from] io::Error),
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

        let bf = cpu.trim();

        let cpu =
            bf.replace("cpu", "");

        if cpu.len() == bf.len() {
            return Err(UsageError::CpuMissing);
        }

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
        ] = parts.flat_map(|x| x.parse::<u64>())
            .collect::<Vec<u64>>().try_into()
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

#[tokio::test]
pub async fn test_probe_cpu() -> crate::Any {
    let (
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
    ) = (
        257859,
        80,
        36625,
        8098834,
        4146,
        0,
        2561,
        1,
        2,
        3,
    );

    let data = format!("cpu  {user} {nice} {system} {idle} {iowait} {irq} {softirq} {steal} {guest} {guest_nice}");

    crate::testing::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let cpu = Cpu::probe().await?;

    assert_eq!(guest_nice, cpu.usage.guest_nice, "invalid cpu nice metric");
    assert_eq!(softirq, cpu.usage.softirq, "invalid cpu nice metric");
    assert_eq!(system, cpu.usage.system, "invalid cpu nice metric");
    assert_eq!(iowait, cpu.usage.iowait, "invalid cpu nice metric");
    assert_eq!(guest, cpu.usage.guest, "invalid cpu nice metric");
    assert_eq!(steal, cpu.usage.steal, "invalid cpu nice metric");
    assert_eq!(user, cpu.usage.user, "invalid cpu user metric");
    assert_eq!(nice, cpu.usage.nice, "invalid cpu nice metric");
    assert_eq!(idle, cpu.usage.idle, "invalid cpu nice metric");
    assert_eq!(irq, cpu.usage.irq, "invalid cpu nice metric");

    Ok(())
}
