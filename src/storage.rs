use std::borrow::Cow;

use google_cloud_default::WithAuthExt;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        delete::DeleteObjectRequest,
        download::Range,
        get::GetObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
};
use serde::Deserialize;
use serenity::{
    prelude::{TypeMapKey},
    Error,
};

pub struct StorageClient {
    pub client: google_cloud_storage::client::Client,
}

impl StorageClient {
    pub async fn new() -> Self {
        let storage_config = ClientConfig::default()
            .with_auth()
            .await
            .unwrap_or_else(|err| {
                println!("{:?}", err);
                panic!("{:?}", err);
            });
        let client = Client::new(storage_config);

        StorageClient { client }
    }

    pub async fn remove(&self, path: String) -> Result<(), Error> {
        let request = DeleteObjectRequest {
            bucket: "ziplod-assets".into(),
            object: path,
            ..Default::default()
        };

        self.client.delete_object(&request).await.map_err(|err| {
            println!("{}", err);
            Error::Other("Failed to delete the player")
        })
    }

    pub async fn download(&self, path: String) -> Result<Vec<u8>, Error> {
        let request = GetObjectRequest {
            bucket: "ziplod-assets".into(),
            object: path,
            ..Default::default()
        };

        let range = Range::default();

        self.client
            .download_object(&request, &range)
            .await
            .map_err(|_| Error::Other("Failed to fetch the specified object"))
    }

    pub async fn remove_json(&self, path: String) -> Result<(), Error> {
        self.remove(path + ".json").await
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

        let save_name = path + ".json";

        let upload_media = Media {
            name: Cow::Owned(save_name),
            content_type: Cow::Borrowed("application/json"),
            content_length: Some(content.len().try_into().unwrap()),
        };

        self.client
            .upload_object(&upload_request, content, &UploadType::Simple(upload_media))
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
