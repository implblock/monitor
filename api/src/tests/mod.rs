#[cfg(test)]
mod network;
#[cfg(test)]
mod memory;
#[cfg(test)]
mod uptime;
#[cfg(test)]
mod cpu;

use tokio::{fs::{create_dir, File}, io::AsyncWriteExt};

pub async fn point_env_dir<S, A>(
    env: S,
    path: S,
    data: A,
) -> crate::Any
where
    S: AsRef<str>,
    A: AsRef<[S]>,
{
    let _ = create_dir(path.as_ref()).await;

    for v in data.as_ref() {
        let _ = create_dir(
            format!("{}/{}", path.as_ref(), v.as_ref())
        ).await;
    }

    std::env::set_var(
        env.as_ref(),
        path.as_ref(),
    );

    Ok(())
}

pub async fn point_env_file<S>(
    env: S,
    path: S,
    data: S,
) -> crate::Any
where
    S: AsRef<str>,
{
    let mut file = File::create(
        path.as_ref(),
    ).await?;

    file.write_all(data.as_ref().as_bytes())
        .await?;

    std::env::set_var(
        env.as_ref(),
        path.as_ref(),
    );

    Ok(())
}
