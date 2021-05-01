use azure_core::prelude::*;
use azure_core::{HttpClient, WasiHttpClient};
use azure_cosmos::prelude::*;
use azure_cosmos::prelude::*;
use azure_cosmos::resources::collection::*;
use azure_cosmos::responses::GetDocumentResponse;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::error::Error;
use std::sync::Arc;

fn main() {
    block_on(list()).unwrap();
    // block_on(create_delete()).unwrap();
}

async fn list() -> Result<(), Box<dyn Error + Send + Sync>> {
    // First we retrieve the account name and master key from environment variables.
    // We expect master keys (ie, not resource constrained)
    let master_key =
        std::env::var("COSMOS_MASTER_KEY").expect("Set env variable COSMOS_MASTER_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    // This is how you construct an authorization token.
    // Remember to pick the correct token type.
    // Here we assume master.
    // Most methods return a ```Result<_, CosmosError>```.
    // ```CosmosError``` is an enum union of all the possible underlying
    // errors, plus Azure specific ones. For example if a REST call returns the
    // unexpected result (ie NotFound instead of Ok) we return an Err telling
    // you that.
    let authorization_token = AuthorizationToken::primary_from_base64(&master_key)?;

    // Once we have an authorization token you can create a client instance. You can change the
    // authorization token at later time if you need, for example, to escalate the privileges for a
    // single operation.
    // Here we are using reqwest but other clients are supported (check the documentation).
    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let client = CosmosClient::new(http_client, account.clone(), authorization_token);

    // The Cosmos' client exposes a lot of methods. This one lists the databases in the specified
    // account. Database do not implement Display but deref to &str so you can pass it to methods
    // both as struct or id.
    let databases = client.list_databases().execute().await?;

    println!(
        "Account {} has {} database(s)",
        account,
        databases.databases.len()
    );

    // try get on the first database (if any)
    if let Some(db) = databases.databases.first() {
        println!("getting info of database {}", &db.id);
        let db = client
            .clone()
            .into_database_client(db.id.clone())
            .get_database()
            .execute()
            .await?;
        println!("db {} found == {:?}", &db.database.id, &db);
    }

    // Each Cosmos' database contains one or more collections. We can enumerate them using the
    // list_collection method.

    for db in databases.databases {
        let database_client = client.clone().into_database_client(db.id.clone());
        let collections = database_client.list_collections().execute().await?;
        println!(
            "database {} has {} collection(s)",
            db.id,
            collections.collections.len()
        );

        for collection in collections.collections {
            println!("\tcollection {}", collection.id);

            let collection_response = database_client
                .clone()
                .into_collection_client(collection.id)
                .get_collection()
                .execute()
                .await?;

            println!("\tcollection_response {:?}", collection_response);
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MySampleStruct<'a> {
    id: Cow<'a, str>,
    a_string: Cow<'a, str>,
    a_number: u64,
    a_timestamp: i64,
}

impl<'a> azure_cosmos::CosmosEntity<'a> for MySampleStruct<'a> {
    type Entity = &'a str;

    fn partition_key(&'a self) -> Self::Entity {
        self.id.as_ref()
    }
}

// This example expects you to have created a collection
// with partitionKey on "id".

async fn create_delete() -> Result<(), Box<dyn Error + Send + Sync>> {
    let master_key =
        std::env::var("COSMOS_MASTER_KEY").expect("Set env variable COSMOS_MASTER_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    let authorization_token = AuthorizationToken::primary_from_base64(&master_key)?;

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(WasiHttpClient {}));
    let client = CosmosClient::new(http_client, account, authorization_token);
    let client = client.into_database_client("wasiwasi-test");
    let client = client.into_collection_client("azuresdkt");

    let mut doc = MySampleStruct {
        id: Cow::Owned(format!("unique_id{}", 501)),
        a_string: Cow::Borrowed("Something here"),
        a_number: 600,
        a_timestamp: chrono::Utc::now().timestamp(),
    };

    // let's add an entity.
    let create_document_response = client
        .create_document()
        .is_upsert(true)
        .execute(&doc)
        .await?;

    println!(
        "create_document_response == {:#?}\n\n\n",
        create_document_response
    );

    let document_client = client
        .clone()
        .into_document_client(doc.id.clone(), &doc.id)?;

    let get_document_response = document_client
        .get_document()
        .consistency_level(&create_document_response)
        .execute::<serde_json::Value>()
        .await?;
    println!("get_document_response == {:#?}", get_document_response);

    let document_client = client.clone().into_document_client("ciccia", &doc.id)?;

    let get_document_response = document_client
        .get_document()
        .consistency_level(&get_document_response)
        .execute::<serde_json::Value>()
        .await?;
    println!(
        "get_document_response == {:#?}\n\n\n",
        get_document_response
    );

    let list_documents_response = client
        .list_documents()
        .consistency_level(&get_document_response)
        .execute::<serde_json::Value>()
        .await?;
    println!("list_documents_response == {:#?}", list_documents_response);

    let query_documents_response = client
        .query_documents()
        .consistency_level(&list_documents_response)
        .query_cross_partition(true)
        .execute::<serde_json::Value, _>("SELECT * FROM c WHERE c.a_number = 600")
        .await?;
    println!(
        "query_documents_response == {:#?}",
        query_documents_response
    );

    doc.a_number = 43;

    let replace_document_response = client
        .into_document_client(doc.id.clone(), &doc.id)?
        .replace_document()
        .consistency_level(&query_documents_response)
        .execute(&doc)
        .await?;
    println!(
        "replace_document_response == {:#?}",
        replace_document_response
    );

    Ok(())
}
