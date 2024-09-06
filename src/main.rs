#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let _ = mtnmomo::MomoCallbackListener::serve("3000".to_string()).await;
    Ok(())
}
