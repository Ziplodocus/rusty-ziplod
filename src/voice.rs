use futures_util::{Stream, StreamExt};
use std::{
    io::{Read, Seek, Write},
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
};
use symphonia::core::{
    io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
    probe::Hint,
};

use serenity::{
    model::prelude::{ChannelId, GuildId},
    prelude::Context,
};
use songbird::input::{AudioStream, Input, LiveInput};

use crate::errors::Error;

pub async fn play(
    ctx: &Context,
    channel_id: ChannelId,
    guild_id: GuildId,
    mut file_stream: impl Stream<Item = Result<u8, Error>> + Unpin,
) -> Result<(), Error> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // Create a channel, ReadableReceiver is a wrapper to allow passing the stream synchronously
    let (tx, rx) = mpsc::channel();

    let media_stream = MediaSourceStream::new(
        Box::new(ReadableReceiver {
            receiver: Mutex::new(rx),
        }),
        MediaSourceStreamOptions::default(),
    );

    // All my audio streams are mp3s
    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let audio_stream = AudioStream {
        input: media_stream,
        hint: Some(hint),
    };

    let input = LiveInput::Wrapped(audio_stream);
    let input = Input::Live(input, None);

    {
        let handler_lock = manager.join(guild_id, channel_id).await.unwrap();
        let mut handler = handler_lock.lock().await;
        handler.play_only_input(input);
    }

    let mut buffer: Vec<u8> = Vec::with_capacity(64);
    while let Some(Ok(byte)) = file_stream.next().await {
        buffer.push(byte);

        if buffer.len() >= 64 {
            let res = tx.send(buffer);
            if let Err(err) = res {
                dbg!(err);
            }

            buffer = Vec::with_capacity(64);
        }
    }

    println!("Finished main writing!");

    if !buffer.is_empty() {
        let res = tx.send(buffer);
        if let Err(err) = res {
            dbg!(err);
        }
    }

    println!("Finished writing!");

    Ok(())
}

struct ReadableReceiver {
    receiver: Mutex<Receiver<Vec<u8>>>,
}

// Simply reads until the receiver has no more vecs to receive
impl Read for ReadableReceiver {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        buf.write(
            &self
                .receiver
                .lock()
                .map_err(|err| {
                    dbg!(err);
                    std::io::Error::other("Failed to get the lock")
                })?
                .recv()
                .map_err(|err| {
                    dbg!(err);
                    dbg!("No more details?");
                    std::io::Error::other("Oh no")
                })?,
        )
    }
}

// Not seekable, so just leaving as a todo
impl Seek for ReadableReceiver {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

// Length isn't known as it's reading from a network stream, nor is it seekable
impl MediaSource for ReadableReceiver {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}
