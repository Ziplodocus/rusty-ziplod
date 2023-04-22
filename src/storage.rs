use std::borrow::Cow;

use google_cloud_default::WithAuthExt;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        download::Range,
        get::GetObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
};
use serde::Deserialize;
use serenity::{
    prelude::{Context, TypeMapKey},
    Error,
};

pub struct StorageClient {
    pub client: google_cloud_storage::client::Client,
}

impl StorageClient {
    pub async fn new() -> Self {
        let storage_config = match ClientConfig::default().with_auth().await {
            Ok(thing) => thing,
            Err(err) => {
                println!("{:?}", err);
                panic!("{:?}", err);
            }
        };
        let client = Client::new(storage_config);

        StorageClient { client }
    }

    pub async fn download(&self, path: String) -> Result<Vec<u8>, Error> {
        let request = GetObjectRequest {
            bucket: "ziplod-assets".into(),
            object: path,
            ..Default::default()
        };

        let range = Range::default();

        self.client
            .download_object(&request, &range, None)
            .await
            .map_err(|_| Error::Other("User does not have a player saved"))
    }

    pub async fn download_json<T: for<'a> Deserialize<'a>>(
        &self,
        path: String,
    ) -> Result<T, Error> {
        let bytes = self.download(path + ".json").await?;

        serde_json::from_slice::<T>(&bytes).map_err(|err| Error::Json(err))
    }

    pub async fn upload_json(&self, path: String, content: String) -> Result<(), Error> {
        let upload_request = UploadObjectRequest {
            bucket: "ziplod-assets".into(),
            ..Default::default()
        };

        let save_name = "zumbor/".to_string() + &path + ".json";

        let upload_media = Media {
            name: Cow::Owned(save_name),
            content_type: Cow::Borrowed("application/json"),
            content_length: Some(content.len()),
        };

        self.client
            .upload_object(
                &upload_request,
                content,
                &UploadType::Simple(upload_media),
                Default::default(),
            )
            .await
            .map_err(|err| {
                dbg!(err);
                Error::Other("Failed to upload object")
            })?;

        Ok(())
    }
}

impl TypeMapKey for StorageClient {
    type Value = StorageClient;
}
