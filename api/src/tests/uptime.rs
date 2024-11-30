use std::time::Duration;

use monitor_core::probe::Probe;

use crate::resources::uptime::{Error, Uptime};

const UPTIME_STR: &str = "10.0 20.0";

const UPTIME: Uptime = Uptime {
    uptime: Duration::new(10, 0),
    idle: Duration::new(20, 0),
};

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_uptime_parse_float() -> crate::Any {
    let data = UPTIME_STR.replace("20.0", "abc");

    super::point_env_file(
        "UPTIME",
        "/tmp/uptime",
        &data,
    ).await?;

    let err = Uptime::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::ParseFloat(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_uptime_invalid() -> crate::Any {
    let data = format!("{} 30.0", UPTIME_STR);

    super::point_env_file(
        "UPTIME",
        "/tmp/uptime",
        &data,
    ).await?;

    let err = Uptime::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::InvalidUptime));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_uptime_success() -> crate::Any {
    let data = UPTIME_STR;

    super::point_env_file(
        "UPTIME",
        "/tmp/uptime",
        data,
    ).await?;

    let uptime = Uptime::probe().await?;

    assert_eq!(uptime, UPTIME);

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_uptime_empty() -> crate::Any {
    let data = "";

    super::point_env_file(
        "UPTIME",
        "/tmp/uptime",
        data,
    ).await?;

    let err = Uptime::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::Empty));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_uptime_io() -> crate::Any {
    std::env::set_var("UPTIME", "/tmp/not/a/file");

    let err = Uptime::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::Io(_)));

    Ok(())
}
