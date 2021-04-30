use azure_core::{HttpClient, WasiHttpClient};
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;
use bytes::{BufMut, Bytes};
use futures::executor::block_on;
use futures::stream::StreamExt;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;

pub fn main() {
    // block_on(run()).unwrap();
    block_on(put()).unwrap();
    block_on(stream()).unwrap();
}

async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let account = std::env::var("STORAGE_ACCOUNT")?;
    let master_key = std::env::var("STORAGE_MASTER_KEY")?;

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let storage_account_client =
        StorageAccountClient::new_access_key(http_client.clone(), &account, &master_key);
    let storage_client = storage_account_client.as_storage_client();
    let blob_client = storage_client
        .as_container_client("wasistorage")
        .as_blob_client("hobbits.txt");

    let response = blob_client.get().execute().await?;
    let mut stream = Box::pin(blob_client.get().stream(32));

    while let Some(value) = stream.next().await {
        println!("received {:?} bytes", value?.data.len());
    }

    let s_content = String::from_utf8(response.data.to_vec())?;
    println!("s_content == {}", s_content);

    Ok(())
}

async fn put() -> Result<(), Box<dyn Error + Send + Sync>> {
    // First we retrieve the account name and master key from environment variables.
    let account =
        std::env::var("STORAGE_ACCOUNT").expect("Set env variable STORAGE_ACCOUNT first!");
    let master_key =
        std::env::var("STORAGE_MASTER_KEY").expect("Set env variable STORAGE_MASTER_KEY first!");

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let storage_account_client =
        StorageAccountClient::new_access_key(http_client.clone(), &account, &master_key);

    let storage_client = storage_account_client.as_storage_client();
    let blob = storage_client
        .as_container_client("wasistorage")
        .as_blob_client("test1");

    // this example fills a 1 KB file with ASCII text and
    // sends it in chunks of 256 bytes (4 chunks).
    // It then finalizes the block blob by calling
    // PutBlockList. Finally it gets back
    // the blob as a whole.
    let mut data = bytes::BytesMut::with_capacity(1 * 1024);
    for _ in 0..1 * (1024 / 64) {
        data.put("the brown fox jumped over the lazy dog. 123456789Pangram12345678".as_bytes());
    }

    let data = data.freeze();
    println!("data to send is {} bytes.", data.len());

    let mut block_ids = Vec::new();
    for i in 0..(1024 / 256) {
        let slice = data.slice(i * 256..(i + 1) * 256);
        let block_id = Bytes::from(format!("{}", i));
        block_ids.push(block_id.clone());
        let hash = md5::compute(slice.clone()).into();
        let put_block_response = blob
            .put_block(block_id, slice)
            .hash(&hash)
            .execute()
            .await?;
        println!("put_block_response == {:#?}", put_block_response);
    }

    let mut block_list = BlockList::default();
    for id in block_ids.into_iter() {
        block_list.blocks.push(BlobBlockType::new_uncommitted(id));
    }

    let res = blob
        .put_block_list(&block_list)
        .content_md5(md5::compute(data))
        .execute()
        .await?;

    println!("PutBlockList == {:?}", res);
    let retrieved_blob = blob.get().execute().await?;
    println!("retrieved_blob == {:?}", retrieved_blob);

    let s = String::from_utf8(retrieved_blob.data.to_vec())?;
    println!("retrieved contents == {}", s);

    Ok(())
}
async fn stream() -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_name = "azure_sdk_for_rust_stream_test.txt";

    // First we retrieve the account name and master key from environment variables.
    let account =
        std::env::var("STORAGE_ACCOUNT").expect("Set env variable STORAGE_ACCOUNT first!");
    let master_key =
        std::env::var("STORAGE_MASTER_KEY").expect("Set env variable STORAGE_MASTER_KEY first!");

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let storage_account_client =
        StorageAccountClient::new_access_key(http_client.clone(), &account, &master_key);
    let storage_client = storage_account_client.as_storage_client();
    let blob = storage_client
        .as_container_client("wasistorage")
        .as_blob_client(file_name);

    let string = "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF";
    let _response = blob
        .put_block_blob(string)
        .content_type("text/plain")
        .execute()
        .await?;
    println!("{}/{} blob created!", "wasistorage", file_name);
    // this is how you stream data from azure blob. Notice that you have
    // to specify the range requested. Also make sure to specify how big
    // a chunk is going to be. Bigger chunks are of course more efficient as the
    // http overhead will be less but it also means you will have to wait for more
    // time before receiving anything. In this example we use a very small chunk size
    // just to make sure to loop at least twice.
    let mut stream = Box::pin(blob.get().stream(128));
    let result = Rc::new(RefCell::new(Vec::new()));
    {
        let mut res_closure = result.borrow_mut();
        while let Some(value) = stream.next().await {
            let mut value = value?.data.to_vec();
            println!("received {:?} bytes", value.len());
            res_closure.append(&mut value);
        }
    }
    let returned_string = {
        let rlock = result.borrow();
        String::from_utf8(rlock.to_vec())?
    };
    // You can of course conctenate all the
    // pieces as shown below.
    // It generally does not make sense as you
    // will lose the ability to process the data as it
    // comes in.
    //
    //let fut = stream.concat2().map(|res| {
    //    println!("all blocks received");
    //    res
    //});
    //
    //let result = reactor.run(fut)?;
    //let returned_string = String::from_utf8(result)?;
    println!("{}", returned_string);
    assert!(
        string == returned_string,
        "string = {}, returned_string = {}",
        string,
        returned_string
    );
    blob.delete()
        .delete_snapshots_method(DeleteSnapshotsMethod::Include)
        .execute()
        .await?;
    Ok(())
}
