use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    process::{Child, Command, Stdio},
    thread::{self, JoinHandle}, rc::Rc, sync::Arc,
};

use serde_json::{Map, Value};

use crate::errors::Error;

pub fn convert(stream: Arc<[u8]>, format: &str) -> Result<(Child, AudioMeta, JoinHandle<Result<(), Error>>), Error> {
    let meta = get_meta(stream.clone())?;

    println!("Converting the audio from mp3...");
    let ffmpeg_args = [
        "-v",
        "error",
        "-f",
        &meta.format_name,
        "-i",
        "pipe:0",
        "-f",
        format,
        "pipe:1",
    ];

    let mut ffmpeg = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = ffmpeg.stdin.take().unwrap();

    let write_handle = thread::spawn(move || -> Result<(), Error> {
        stdin.write_all(&stream)?;
        stdin.flush()?;
        println!("Written all to buffer.");
        Ok(())
    });

    return Ok((ffmpeg, meta, write_handle));
}

fn get_meta(stream: Arc<[u8]>) -> Result<AudioMeta, Error> {
    println!("Getting the audio meta");
    let mut cmd = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-f",
            "mp3",
            "-i",
            "pipe:0",
            "-select_streams",
            "a:0",
            "-show_streams",
            "-show_format",
            // "-show_entries",
            // "stream=sample_rate,duration,codec_name",
            "-of",
            "json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = cmd.stdin.take().unwrap();
    let stdout = cmd.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let read_thread = thread::spawn(|| -> Result<Map<String, Value>, Error> {
        let map: Map<String, Value> = serde_json::from_reader(reader)?;
        Ok(map)
    });

    let _writer_handle = thread::spawn(move || -> Result<(), Error> {
        stdin.write_all(&stream)?;
        stdin.flush()?;
        Ok(())
    });

    let map = read_thread.join().expect("This to work :(")?;

    let format = map
        .get("format")
        .expect("Format is returned")
        .as_object()
        .unwrap();

    let main_stream = map
        .get("streams")
        .expect("There is a stream")
        .as_array()
        .unwrap()
        .get(0)
        .expect("Uh oh there was no stream");

    let sample_rate: u32 = main_stream
        .get("sample_rate")
        .expect("There is a sample rate")
        .as_str()
        .unwrap()
        .parse()
        .expect("Is convertible");

    let channels: u64 = main_stream
        .get("channels")
        .expect("Channels is determined")
        .as_u64()
        .unwrap();

    let channel_layout = main_stream
        .get("channel_layout")
        .expect("Stereo/mono is determined")
        .as_str()
        .expect("Is convertible");

    let is_stereo = channel_layout == "stereo";

    let format_name = format
        .get("format_name")
        .expect("Format is determined in audio file")
        .as_str()
        .expect("Is convertible");

    let bit_rate: u32 = format
        .get("bit_rate")
        .expect("Bitrate is determined in audio file")
        .as_str()
        .unwrap()
        .parse()
        .expect("Is convertible");

    let bit_rate = bit_rate / 1000;
    let sample_rate = sample_rate / 1000;

    println!("Determined the audio meta");

    Ok(AudioMeta {
        format_name: format_name.into(),
        bit_rate: (bit_rate.to_string() + "k").into(),
        sample_rate: (sample_rate.to_string() + "k").into(),
        channels: channels.into(),
        is_stereo,
    })
}

#[derive(Debug, Clone)]
pub struct AudioMeta {
    format_name: Box<str>,
    bit_rate: Box<str>,
    sample_rate: Box<str>,
    channels: u64,
    pub is_stereo: bool,
}
