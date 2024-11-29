use tokio::{fs::File, io::AsyncWriteExt};

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
