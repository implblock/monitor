use std::fs::remove_dir_all;

use crate::resources::cpu::{Core, CoreError, Cores, Info, InfoError, Usage, UsageError};
use monitor::probe::Probe;
use tokio::{fs::File, io::AsyncWriteExt};

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
pub async fn test_probe_cpu_usage_invalid_cpu_line() -> crate::Any {
    let data = format!("{} 0 0 0 0 0", USAGE_STR);

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let err = Usage::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::InvalidCpuLine));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_usage_parse_int() -> crate::Any {
    let data = USAGE_STR.replace("10", "abc");

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let err = Usage::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::ParseInt(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_usage_missing() -> crate::Any {
    let data = USAGE_STR.replace("cpu", "");

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        &data,
    ).await?;

    let err = Usage::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::CpuMissing));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_usage_success() -> crate::Any {
    let data = USAGE_STR;

    super::point_env_file(
        "STAT",
        "/tmp/stat",
        data,
    ).await?;

    let usage = Usage::probe().await?;

    assert_eq!(usage, USAGE);

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_usage_io() -> crate::Any {
    std::env::set_var("STAT", "/tmp/not/a/file");

    let err = Usage::probe().await
        .unwrap_err();

    assert!(matches!(err, UsageError::Io(_)));

    Ok(())
}

struct SetDown;

impl Drop for SetDown {
    fn drop(&mut self) {
        remove_dir_all("/tmp/coretemp").unwrap();
    }
}

async fn setup(
    crit_alarm: impl AsRef<str>,
    label: impl AsRef<str>,
) -> anyhow::Result<SetDown> {
    super::point_env_dir(
        "CORETEMP",
        "/tmp/coretemp",
        []
    ).await?;

    async fn write_to(
        data: impl AsRef<str>,
        path: impl AsRef<str>
    ) -> crate::Any {
        let mut file = File::create(path.as_ref())
            .await?;

        file.write_all(
            data.as_ref().as_bytes()
        ).await?;

        Ok(())
    }

    let coretemp = "/tmp/coretemp";

    write_to(
        crit_alarm.as_ref(),
        format!("{}/temp1_crit_alarm", coretemp),
    ).await?;

    write_to(
        "47000",
        format!("{}/temp1_input", coretemp),
    ).await?;

    write_to(
        "100000",
        format!("{}/temp1_crit", coretemp),
    ).await?;

    write_to(
        label.as_ref(),
        format!("{}/temp1_label", coretemp),
    ).await?;

    write_to(
        "100000",
        format!("{}/temp1_max", coretemp),
    ).await?;

    Ok(SetDown)
}

fn cores() -> Vec<Core> {
    vec![
        Core {
            max_temp: 100000,
            crit: 100000,
            temp: 47000,
            count: 0,
        }
    ]
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_cores_invalid_label() -> crate::Any {
    let _setdown = setup("0", "Core").await?;

    let err = Cores::probe().await
        .unwrap_err();

    assert!(matches!(err, CoreError::InvalidLabel(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_cores_parse_int() -> crate::Any {
    let _setdown = setup("0", "Core a").await?;

    let err = Cores::probe().await
        .unwrap_err();

    assert!(matches!(err, CoreError::ParseInt(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_cores_success() -> crate::Any {
    let _setdown = setup("0", "Core 0").await?;

    let res = Cores::probe().await?;

    assert_eq!(cores(), res);

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_cores_io() -> crate::Any {
    std::env::set_var("CORETEMP", "/tmp/not/a/file");

    let err = Cores::probe().await
        .unwrap_err();

    assert!(matches!(err, CoreError::Io(_)));

    Ok(())
}

const INFO_STR: &str = "\
processor : 0
vendor_id  : GenuineIntel
model name  : Intel(R) Core(TM) i7-10870H CPU @ 2.20GHz
cpu MHz  : 800.000
siblings  : 16
cpu cores : 8
power management:
";

fn info() -> Info {
    Info {
        model: Some(String::from(
            "Intel(R) Core(TM) i7-10870H CPU @ 2.20GHz"
        )),
        mhz: 800.000,
        siblings: 16,
        cores: 8,
    }
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_parse_float() -> crate::Any {
    let data = INFO_STR.replace("800.000", "abc");

    super::point_env_file(
        "INFO",
        "/tmp/info",
        &data,
    ).await?;

    let err = Info::probe().await
        .unwrap_err();

    assert!(matches!(err, InfoError::ParseFloat(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_missing_mhz() -> crate::Any {
    let data = INFO_STR.replace("cpu MHz", "placeholder");

    super::point_env_file(
        "INFO",
        "/tmp/info",
        &data,
    ).await?;

    let err = Info::probe().await
        .unwrap_err();

    dbg!(&err);

    assert!(matches!(err, InfoError::CpuMhzNotFound));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_missing_cores() -> crate::Any {
    let data = INFO_STR.replace("cpu cores", "placeholder");

    super::point_env_file(
        "INFO",
        "/tmp/info",
        &data,
    ).await?;

    let err = Info::probe().await
        .unwrap_err();

    dbg!(&err);

    assert!(matches!(err, InfoError::CoresNotFound));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_missing_siblings() -> crate::Any {
    let data = INFO_STR.replace("siblings", "placeholder");

    super::point_env_file(
        "INFO",
        "/tmp/info",
        &data,
    ).await?;

    let err = Info::probe().await
        .unwrap_err();

    dbg!(&err);

    assert!(matches!(err, InfoError::SiblingsNotFound));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_parse_int() -> crate::Any {
    let data = INFO_STR.replace("8", "abc");

    super::point_env_file(
        "INFO",
        "/tmp/info",
        &data,
    ).await?;

    let err = Info::probe().await
        .unwrap_err();

    assert!(matches!(err, InfoError::ParseInt(_)));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_success() -> crate::Any {
    super::point_env_file(
        "INFO",
        "/tmp/info",
        INFO_STR,
    ).await?;

    let res = Info::probe().await?;

    assert_eq!(res, info());

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_cpu_info_io() -> crate::Any {
    std::env::set_var("INFO", "/tmp/not/a/file");

    let err = Info::probe().await
        .unwrap_err();

    assert!(matches!(err, InfoError::Io(_)));

    Ok(())
}
