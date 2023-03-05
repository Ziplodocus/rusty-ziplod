use google_cloud_storage::client::Client;
use google_cloud_storage::client::ClientConfig;
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use google_cloud_storage::sign::SignedURLOptions;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::http::Error;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use rand::seq::SliceRandom;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

// use google_cloud_default::WithAuthExt;
// let config = ClientConfig::default().with_auth().await?;
async fn get_storage_client(config: ClientConfig) -> Result<(), Error> {

    // Create client.
    let mut client = Client::new(config);

    // Upload the file
    let upload_type = UploadType::Simple(Media::new("file.png"));
    let uploaded = client.upload_object(&UploadObjectRequest {
        bucket: "bucket".to_string(),
        ..Default::default()
    }, "hello world".as_bytes(), &upload_type, None).await;

    // Download the file
    let data = client.download_object(&GetObjectRequest {
        bucket: "bucket".to_string(),
        object: "file.png".to_string(),
        ..Default::default()
   }, &Range::default(), None).await;

    // Create signed url.
    let url_for_download = client.signed_url("bucket", "foo.txt", SignedURLOptions::default());
    let url_for_upload = client.signed_url("bucket", "foo.txt", SignedURLOptions {
        method: SignedURLMethod::PUT,
        ..Default::default()
    });
    Ok(())
}

async fn get_ziplod_bucket(client: Client) {

}