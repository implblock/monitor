pub mod routes;

// for simple empty results
type Any = anyhow::Result<()>;

#[tokio::main]
async fn main() -> Any {
    Ok(())
}
