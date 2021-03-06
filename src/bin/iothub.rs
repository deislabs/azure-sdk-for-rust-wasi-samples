use azure_core::{HttpClient, WasiHttpClient};
use futures::executor::block_on;
use iothub::service::resources::{AuthenticationMechanism, DesiredCapability, Status};
use iothub::service::ServiceClient;
use std::error::Error;
use std::sync::Arc;

fn main() {
    block_on(create_delete()).unwrap();
}

async fn create_delete() -> Result<(), Box<dyn Error + Send + Sync>> {
    let iothub_connection_string = std::env::var("IOTHUB_CONNECTION_STRING")
        .expect("Set env variable IOTHUB_CONNECTION_STRING first!");

    let device_id = String::from("wasi-device");

    println!("Getting device twin for device '{}'", device_id);
    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let service_client =
        ServiceClient::from_connection_string(http_client, iothub_connection_string, 3600)?;
    let device = service_client
        .create_device_identity()
        .execute(
            &device_id,
            Status::Enabled,
            AuthenticationMechanism::new_using_symmetric_key(
                "QhgevIUBSWe37q1MP+M/vtktjOcrE74BVbpcxlLQw58=",
                "6YS6w5wqkpdfkEW7iOP1NvituehFlFRfPko2n7KY4Gk",
            ),
        )
        .await?;

    println!("Successfully created a new device '{}'", device.device_id);

    println!(
        "Setting status to disabled and set IoT Edge capability of device '{}'",
        device.device_id
    );
    let device = service_client
        .update_device_identity(device.etag)
        .device_capability(DesiredCapability::IotEdge)
        .execute(
            &device_id,
            Status::Enabled,
            AuthenticationMechanism::new_using_symmetric_key(
                "QhgevIUBSWe37q1MP+M/vtktjOcrE74BVbpcxlLQw58=",
                "6YS6w5wqkpdfkEW7iOP1NvituehFlFRfPko2n7KY4Gk",
            ),
        )
        .await?;

    println!("Getting device identity of '{}'", device.device_id);
    let device = service_client.get_device_identity(device.device_id).await?;
    println!("Identity is: {:?}", device);

    println!("Deleting device '{}'", device.device_id);
    service_client
        .delete_device_identity(device.device_id, device.etag)
        .execute()
        .await?;

    Ok(())
}
