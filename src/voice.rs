use futures_util::{Stream, StreamExt};
use std::{
    io::{Read, Seek, Write},
    sync::{
        Mutex,
        mpsc::{self, Receiver},
    },
};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};

use serenity::model::prelude::{ChannelId, GuildId};
use songbird::input::{AudioStream, Input, LiveInput};

use crate::{Context, errors::Error};

pub async fn play(
    ctx: Context<'_>,
    channel_id: ChannelId,
    guild_id: GuildId,
    mut file_stream: impl Stream<Item = Result<u8, Error>> + Unpin,
) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
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
    let audio_stream = AudioStream {
        input: media_stream,
    };

    let input = LiveInput::Wrapped(audio_stream);
    let input = Input::Live(input, None);

    {
        let handler_lock = match manager.join(guild_id, channel_id).await {
            Ok(handler) => handler,
            Err(err) => {
                dbg!(&err);
                return Err(Error::Songbird(err));
            }
        };
        let mut handler = handler_lock.lock().await;
        handler.play_only_input(input);
    }

    while let Some(Ok(byte)) = file_stream.next().await {
        let res = tx.send(byte);
        if let Err(err) = res {
            dbg!(err);
        }
    }

    println!("Finished writing!");

    Ok(())
}

struct ReadableReceiver {
    receiver: Mutex<Receiver<u8>>,
}

// Simply reads until the receiver has no more vecs to receive
impl Read for ReadableReceiver {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        buf.write(&[self
            .receiver
            .lock()
            .expect("Should only be the one lock. One reader.")
            .recv()
            .map_err(|_| std::io::Error::other("Reading finished"))?])
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
