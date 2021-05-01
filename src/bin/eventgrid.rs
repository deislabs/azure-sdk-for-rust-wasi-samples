use azure_core::{HttpClient, WasiHttpClient};
use azure_event_grid::{Event, EventGridClient};
use futures::executor::block_on;
use serde::Serialize;
use std::env;
use std::sync::Arc;

fn main() {
    block_on(run()).unwrap();
}

#[derive(Serialize)]
struct Data {
    number: i32,
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let topic_host_name =
        env::var("TOPIC_HOST_NAME").expect("Missing TOPIC_HOST_NAME environment variable.");
    let topic_key = env::var("TOPIC_KEY").expect("Missing TOPIC_KEY environment variable.");

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let client = EventGridClient::new(topic_host_name, topic_key, http_client);
    let event = Event::<Data>::new(
        None,
        "ACME.Data.DataPointCreated",
        "/acme/data",
        Data { number: 42 },
        None,
    );

    client.publish_events(&[event]).await?;
    Ok(())
}
