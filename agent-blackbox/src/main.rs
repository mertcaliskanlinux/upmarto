#[tokio::main]
async fn main() {
    upmarto::run().await.expect("Upmarto server error");
}
