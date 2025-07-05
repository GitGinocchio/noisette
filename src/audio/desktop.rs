use rodio::OutputStreamHandle;
#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, OutputStream, Sink};
use std::{fs::File, io::BufReader, sync::{Arc, Mutex}};

use crate::Sound;

use super::interface::AudioBackend;

pub struct DesktopAudio {
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle
}

impl DesktopAudio {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            sink: Arc::new(Mutex::new(None)),
            _stream,
            stream_handle
        }
    }
}

impl AudioBackend for DesktopAudio {
    fn play(&mut self, sound: &Sound) {
        self.stop(); // Stop any currently playing sound

        //sound.playing = true;

        // Safely unwrap the path
        let path = match &sound.path {
            Some(p) => p.clone(),
            None => {
                eprintln!("Sound path is None!");
                return;
            }
        };

        // Load and play the sound
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        let file = File::open(path).unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();
        sink.append(source);

        // Store the sink
        *self.sink.lock().unwrap() = Some(sink);
    }

    fn stop(&mut self) {
        // Ferma la riproduzione del Sink se esiste
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
    }

    fn is_playing(&self) -> bool {
        self.sink.lock().unwrap().as_ref().map_or(false, |s| !s.empty())
    }
}
