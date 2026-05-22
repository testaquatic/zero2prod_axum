use zero2prod_axum::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    startup::run().await
}
