use zero2prod_axum::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    run().await
}
