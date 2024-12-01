use std::fs::remove_dir_all;

use tokio::{fs::File, io::AsyncWriteExt};
use monitor::probe::Probe;

use crate::resources::network::{Error, Info, Interface, Network, OperState, Stats};

struct SetDown;

impl Drop for SetDown {
    fn drop(&mut self) {
        remove_dir_all("/tmp/net").unwrap();
    }
}

async fn setup(
    operstate: OperState,
    stat: &str,
    miss_stat: bool,
    miss_info: bool,
    null_speed: bool,
) -> anyhow::Result<SetDown> {
    super::point_env_dir(
        "NET",
        "/tmp/net",
        [
            "testing",
            "testing/statistics"
        ]
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

    let statistics = "/tmp/net/testing/statistics";
    let info = "/tmp/net/testing";

    write_to(
        stat,
        format!("{statistics}/rx_packets"),
    ).await?;

    if !miss_stat {
        write_to(
            "2",
            format!("{statistics}/tx_packets"),
        ).await?;
    }

    write_to(
        "3",
        format!("{statistics}/rx_bytes"),
    ).await?;

    write_to(
        "4",
        format!("{statistics}/tx_bytes"),
    ).await?;

    if !miss_info {
        write_to(
            "testaddr",
            format!("{info}/address"),
        ).await?;
    }

    write_to(
        operstate.to_string(),
        format!("{info}/operstate"),
    ).await?;

    let speed = if null_speed {
        ""
    } else {
        "1000"
    };

    write_to(
        speed,
        format!("{info}/speed"),
    ).await?;

    Ok(SetDown)
}

fn network() -> Network {
    let interfaces = vec![
        Interface {
            path: "/tmp/net/testing".into(),
            name: "testing".to_string(),
            stats: Stats {
                rx_packets: 1,
                tx_packets: 2,
                rx_bytes: 3,
                tx_bytes: 4,
            },
            info: Info {
                address: "testaddr".to_string(),
                operstate: OperState::Up,
                speed: 1000,
            },
        },
    ];

    Network {
        interfaces,
    }
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_null_speed_link_down() -> crate::Any {
    let _setdown = setup(OperState::Down, "1", false, false, true).await?;

    let nw = Network::probe().await?;

    let testing = nw.interfaces.get(0)
        .expect("testing interface doesn't exist");

    assert_eq!(testing.info.speed, 0);

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_null_speed_link_up() -> crate::Any {
    let _setdown = setup(OperState::Up, "1", false, false, true).await?;

    let err = Network::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::InvalidLinkState { .. }));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_missing_stat() -> crate::Any {
    let _setdown = setup(OperState::Up, "1", true, false, false).await?;

    let err = Network::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::MissingStat { .. }));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_missing_info() -> crate::Any {
    let _setdown = setup(OperState::Up, "1", false, true, false).await?;

    let err = Network::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::MissingInfo { .. }));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_success() -> crate::Any {
    let _setdown = setup(OperState::Up, "1", false, false, false).await?;

    let nw = Network::probe().await?;

    assert_eq!(nw, network());

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_parse() -> crate::Any {
    let _setdown = setup(OperState::Up, "abc", false, false, false).await?;

    let err = Network::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::ParseError { .. }));

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
pub async fn test_probe_network_io() -> crate::Any {
    std::env::set_var("NET", "/tmp/not/a/dir");

    let err = Network::probe().await
        .unwrap_err();

    assert!(matches!(err, Error::Io(_)));

    Ok(())
}
