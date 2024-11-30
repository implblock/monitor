use monitor_core::probe::Probe;

use crate::resources::memory::{Error, Memory};

const MEMINFO_STR: &str = 
"Inactive(anon):     1 kB\n\
Active(anon):       2 kB\n\
Inactive(file):     3 kB\n\
Active(file):       4 kB\n\
Unevictable:        5 kB\n\
SwapCached:         6 kB\n\
MemAvailable:       7 kB\n\
SwapTotal:      8 kB\n\
SwapFree:       9 kB\n\
Inactive:       10 kB\n\
Mlocked:        11 kB\n\
Buffers:        12 kB\n\
MemFree:        13 kB\n\
MemTotal:      14 kB\n\
Cached:     15 kB\n\
Active:     16 kB\n";

// Dirty:      17 kB

const MEMINFO: Memory = Memory {
    inactive_anonymous: 1,
    active_anonymous: 2,
    inactive_file: 3,
    active_file: 4,
    unevictable: 5,
    swap_cached: 6,
    available: 7,
    swap_total: 8,
    swap_free: 9,
    inactive: 10,
    m_locked: 11,
    buffers: 12,
    free: 13,
    total: 14,
    cached: 15,
    active: 16,
    dirty: 17,
};

// NOTE:
// make sure dirty's value is 17 when eq'ing it

#[allow(unused)]
use tokio_test::*;

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_invalid_format() -> crate::Any {
    let data = format!("{}Dirty:     17 mB", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::InvalidFormat(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_missing_field() -> crate::Any {
    let data = format!("{}", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::MissingField(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_missing_colon() -> crate::Any {
    let data = format!("{}Dirty     17 kB", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::MissingColon(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_value_empty() -> crate::Any {
    let data = format!("{}Dirty:", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::ValueEmpty(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_parse_int() -> crate::Any {
    let data = format!("{}Dirty:     abc kB", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::ParseInt(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_key_empty() -> crate::Any {
    let data = format!("{}:     17 kB", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::KeyEmpty(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_success() -> crate::Any {
    let data = format!("{}Dirty:    17 kB", MEMINFO_STR);

    super::point_env_file(
        "MEMINFO",
        "/tmp/meminfo",
        &data,
    ).await?;

    let mem = Memory::probe().await?;

    assert_eq!(mem, MEMINFO);

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_mem_io() -> crate::Any {
    std::env::set_var("MEMINFO", "/tmp/not/a/file");

    let err = Memory::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::Io(_)));

    Ok(())
}
