use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use crate::{
    audio_conversion::{convert, get_meta},
    storage::StorageClient,
};
use cloud_storage::Object;

pub async fn add_stereo_meta_information(storage_client: &StorageClient) {
    println!("Fetching objects...");
    for mut object in storage_client
        .fetch_objects(&("tracks/".to_owned()))
        .await
        .unwrap()
    {
        if object.content_type.as_ref().unwrap() != "audio/mpeg" {
            continue;
        }
        if object
            .metadata
            .as_ref()
            .is_some_and(|map| map.get("is_stereo").is_some())
        {
            continue;
        }

        println!("Working on details for {}", &object.name);
        let stream = storage_client.download(&object.name).await.unwrap();

        let meta = get_meta(stream.into()).unwrap();

        match object.metadata.as_mut() {
            Some(map) => {
                map.insert("is_stereo".to_string(), bool_to_string(meta.is_stereo));
            }
            None => {
                object.metadata = Some(HashMap::from([(
                    "is_stereo".to_string(),
                    bool_to_string(meta.is_stereo),
                )]));
            }
        }

        fn bool_to_string(bool: bool) -> String {
            if bool {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }

        storage_client
            .client
            .object()
            .update(&object)
            .await
            .unwrap();

        println!("Metadata updated!");
    }
}
