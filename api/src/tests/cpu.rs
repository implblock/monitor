use crate::resources::cpu::{Cpu, Usage, UsageError};
use monitor::probe::Probe;

const USAGE_STR: &str = "cpu  1 2 3 4 5 6 7 8 9 10";

const USAGE: Usage = Usage {
    user: 1,
    nice: 2,
    system: 3,
    idle: 4,
    iowait: 5,
    irq: 6,
    softirq: 7,
    steal: 8,
    guest: 9,
    guest_nice: 10,
};

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_invalid_cpu_line() -> crate::Any {
    let data = format!("{} 0 0 0 0 0", USAGE_STR);

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let err = Cpu::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::InvalidCpuLine));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_parse_int() -> crate::Any {
    let data = USAGE_STR.replace("10", "abc");

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let err = Cpu::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::ParseInt(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_missing() -> crate::Any {
    let data = USAGE_STR.replace("cpu", "");

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let err = Cpu::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::CpuMissing));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_success() -> crate::Any {
    let data = USAGE_STR;

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        data,
    ).await?;

    let cpu = Cpu::probe().await?;

    assert_eq!(cpu.usage, USAGE);

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_io() -> crate::Any {
    std::env::set_var("STAT", "/tmp/not/a/file");

    let err = Cpu::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::Io(_)));

    Ok(())
}
