use std::sync::Arc;

use crate::errors::Error;

use bytes::Bytes;
use cloud_storage::{Client, ListRequest, Object};
use rand::seq::SliceRandom;
use serde::Deserialize;
use serenity::{
    client::Context,
    futures::{Stream, StreamExt},
    prelude::TypeMapKey,
};
use songbird::typemap::TypeMap;
use tokio::sync::RwLockReadGuard;

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

    pub async fn delete(&self, path: &str) -> Result<(), Error> {
        self.client
            .object()
            .delete(&self.bucket_name, path)
            .await
            .map_err(|err| {
                println!("{}", err);
                Error::Plain("Failed to remove file")
            })
    }

    pub async fn get_stream(
        &self,
        path: &str,
    ) -> Result<impl Stream<Item = Result<u8, cloud_storage::Error>> + Unpin, Error> {
        let object = self.client.object();
        let maybe_obj = object.download_streamed(&self.bucket_name, path);
        println!("Streaming object.");
        maybe_obj.await.map_err(|o| o.into())
    }

    pub async fn get(&self, path: &str) -> Result<Vec<u8>, Error> {
        let object = self.client.object();
        let maybe_obj = object.download(&self.bucket_name, path).await;
        println!("Downloaded object.");
        maybe_obj.map_err(|o| o.into())
    }

    pub async fn is_stereo(&self, path: &str) -> Result<Option<bool>, Error> {
        let obj = self.client.object().read(&self.bucket_name, path).await?;
        match obj.metadata {
            Some(map) => match map.get("is_stereo") {
                Some(val) => match val.as_str() {
                    "true" => Ok(Some(true)),
                    "false" => Ok(Some(false)),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub async fn delete_json(&self, path: &str) -> Result<(), Error> {
        self.delete(&(path.to_owned() + ".json")).await
    }

    pub async fn get_json<T: for<'a> Deserialize<'a>>(&self, path: &str) -> Result<T, Error> {
        let bytes = self.get(&(path.to_owned() + ".json")).await?;

        serde_json::from_slice::<T>(&bytes).map_err(Error::Json)
    }

    pub async fn create(
        &self,
        content: impl Into<Vec<u8>>,
        path: &str,
        mime_type: &str,
    ) -> Result<(), Error> {
        self.client
            .object()
            .create(&self.bucket_name, content.into(), path, mime_type)
            .await?;
        Ok(())
    }

    /**
     *
     */
    pub async fn create_stream(
        &self,
        stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + Sync + 'static,
        path: &str,
        length: impl Into<Option<u64>>,
        mime_type: &str,
    ) -> Result<(), Error> {
        let res = self
            .client
            .object()
            .create_streamed(&self.bucket_name, stream, length, path, mime_type)
            .await;

        if let Err(err) = res {
            dbg!(&err);
        }

        Ok(())
    }

    pub async fn create_json(&self, path: &str, content: String) -> Result<(), Error> {
        self.create(content, path, "application/json").await
    }

    pub async fn get_random(&self, prefix: &str) -> Result<Vec<u8>, Error> {
        let objects = self.get_objects(prefix).await?;

        println!("{:?}", objects);

        let object = objects
            .choose(&mut rand::thread_rng())
            .expect("The random number generated not to exceed the number of objects");

        self.get(&object.name).await
    }

    pub async fn get_count(&self, prefix: &str) -> Result<usize, Error> {
        let objs = self.get_objects(prefix).await?;
        Ok(objs.len())
    }

    pub async fn get_objects(&self, prefix: &str) -> Result<Vec<Object>, Error> {
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

        let items = match Box::pin(list).next().await {
            Some(list) => list?.items,
            None => Vec::new(),
        };

        Ok(items)
    }
}

impl TypeMapKey for StorageClient {
    type Value = StorageClient;
}

pub enum MimeType {
    MP3,
    JSON,
}

impl MimeType {
    fn to_boxed_str(self: &Self) -> Box<str> {
        match self {
            Self::MP3 => "mp3".into(),
            Self::JSON => "json".into(),
        }
    }
}

struct StorageManager<'a> {
    client: &'a StorageClient,
    prefix: Box<str>,
    mime: MimeType,
}

impl StorageManager<'_> {
    async fn get(self: &Self, name: &str) -> Result<Vec<u8>, Error> {
        self.client.get(self.get_full_path(name).as_str()).await
    }

    async fn create(self: &Self, name: &str, content: Vec<u8>) -> Result<(), Error> {
        self.client
            .create(
                content,
                self.get_full_path(name).as_str(),
                &self.mime.to_boxed_str(),
            )
            .await
    }

    async fn delete(self: &Self, name: &str) -> Result<(), Error> {
        self.client.delete(self.get_full_path(name).as_str()).await
    }

    fn get_full_path(self: &Self, name: &str) -> String {
        self.prefix.to_string() + name + self.mime.to_boxed_str().as_ref()
    }
}
