use ch17_async_sensors::run_simulation;

#[tokio::main]
async fn main() {
    run_simulation().await;
}
