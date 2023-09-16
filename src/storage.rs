use crate::errors::Error;

use cloud_storage::{Client, ListRequest, Object};
use rand::seq::SliceRandom;
// use google_cloud_default::WithAuthExt;
// use google_cloud_storage::{
//     client::{Client, ClientConfig},
//     http::objects::{
//         delete::DeleteObjectRequest,
//         download::Range,
//         get::GetObjectRequest,
//         upload::{Media, UploadObjectRequest, UploadType},
//     },
// };
use serde::Deserialize;
use serenity::{
    futures::{future, Stream, StreamExt},
    prelude::TypeMapKey,
};

#[derive(Debug)]
pub struct StorageClient {
    pub client: cloud_storage::Client,
    pub bucket: cloud_storage::Bucket,
    pub bucket_name: String,
}

impl StorageClient {
    pub async fn new(bucket_name: String) -> Self {
        let client = Client::new();
        let bucket = client
            .bucket()
            .read(&bucket_name)
            .await
            .expect("Bucket Success");
        StorageClient {
            client,
            bucket,
            bucket_name,
        }
    }

    pub async fn remove(&self, path: &String) -> Result<(), Error> {
        self.client
            .object()
            .delete(&self.bucket_name, path)
            .await
            .map_err(|err| {
                println!("{}", err);
                Error::Plain("Failed to delete the player")
            })
    }

    pub async fn download_stream(
        &self,
        path: &String,
    ) -> Result<impl Stream<Item = Result<u8, cloud_storage::Error>> + Unpin, Error> {
        let object = self.client.object();
        let maybe_obj = object.download_streamed(&self.bucket_name, &path);
        println!("Streaming object.");
        maybe_obj.await.map_err(|o| o.into())
    }

    pub async fn download(&self, path: &String) -> Result<Vec<u8>, Error> {
        let object = self.client.object();
        let maybe_obj = object.download(&self.bucket_name, &path);
        println!("Downloaded object.");
        maybe_obj.await.map_err(|o| o.into())
    }

    pub async fn remove_json(&self, path: String) -> Result<(), Error> {
        self.remove(&(path + ".json")).await
    }

    pub async fn download_json<T: for<'a> Deserialize<'a>>(
        &self,
        path: String,
    ) -> Result<T, Error> {
        let bytes = self.download(&(path + ".json")).await?;

        serde_json::from_slice::<T>(&bytes).map_err(|err| Error::Json(err))
    }

    pub async fn upload(&self, content: String, path: &str, mime_type: &str) -> Result<(), Error> {
        self.client
            .object()
            .create(&self.bucket_name, content.into(), path, mime_type)
            .await?;
        Ok(())
    }

    pub async fn upload_json(&self, path: &str, content: String) -> Result<(), Error> {
        self.upload(content, path, "application/json").await
    }

    pub async fn download_random(&self, prefix: &str) -> Result<Vec<u8>, Error> {
        let objects = self.fetch_objects(prefix).await?;

        println!("{:?}", objects);

        let object = objects.choose(&mut rand::thread_rng()).unwrap();

        self.download(&object.name).await
    }


    pub async fn fetch_count(&self, prefix: &str) -> Result<usize, Error>{
        let objs = self.fetch_objects(prefix).await?;
        return Ok(objs.len());
    }

    async fn fetch_objects(&self, prefix: &str) -> Result<Vec<Object>, Error>{
        let list = self
            .client
            .object()
            .list(
                &self.bucket_name,
                ListRequest {
                    prefix: Some(prefix.to_owned()),
                    max_results: Some(1000),
                    ..Default::default()
                },
            )
            .await?;

        let mut objects = vec![];
        list.for_each(|list| {
            let mut items = list.unwrap().items;
            objects.append(&mut items);
            future::ready(())
        })
        .await;

        Ok(objects)
    }
}

impl TypeMapKey for StorageClient {
    type Value = StorageClient;
}
