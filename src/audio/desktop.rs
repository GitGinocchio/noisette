use rodio::cpal::{self, Device};
use rodio::cpal::traits::{HostTrait, DeviceTrait};
use rodio::OutputStreamHandle;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::exists;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{fs::File, io::BufReader, sync::{Arc, Mutex}};
use std::sync::mpsc::{self, Sender, Receiver};
use std::collections::HashMap;
use std::thread;

use crate::Sound;

use super::interface::AudioBackend;

// Api

pub struct DesktopAudio {
    sinks: Arc<Mutex<HashMap<String, Arc<Sink>>>>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl DesktopAudio {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            sinks: Arc::new(Mutex::new(HashMap::new())),
            _stream,
            stream_handle,
        }
    }

    pub fn clean_finished_sinks(&self) {
        let mut sinks = self.sinks.lock().unwrap();
        sinks.retain(|_, s| !s.empty());
    }

    pub fn stop_all(&mut self) {
        let mut sinks = self.sinks.lock().unwrap();
        for (_, sink) in sinks.drain() {
            sink.stop();
        }
    }

    pub fn set_device(&mut self, device_name: Option<String>) {
        if let Some(device) = get_device_from_name(device_name) {
            if let Ok((_new_stream, new_handle)) = OutputStream::try_from_device(&device) {
                // Ferma tutti i sink esistenti
                if let Ok(mut sinks) = self.sinks.lock() {
                    for (_, sink) in sinks.iter_mut() {
                        sink.stop();
                    }
                    sinks.clear(); // rimuovi tutti i sink
                }

                // Aggiorna lo stream e il device
                self._stream = _new_stream;
                self.stream_handle = new_handle;
            } else {
                eprintln!("Impossibile cambiare dispositivo audio");
            }
        } else {
            eprintln!("Dispositivo non trovato");
        }
    }
}

impl AudioBackend for DesktopAudio {
    fn play(&mut self, sound: &Sound) {
        self.clean_finished_sinks();

        let path_string = match &sound.path {
            Some(p) => p.clone(),
            None => {
                eprintln!("Sound path is None!");
                return;
            }
        };

        let path = Path::new(&path_string);
        if !exists(path).expect("An error occurred") { return; }

        let file = File::open(&path).unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();

        let sink = Arc::new(Sink::try_new(&self.stream_handle).unwrap());
        sink.as_ref().append(source);

        let mut sinks = self.sinks.lock().unwrap();
        sinks.insert(path_string, sink);
    }

    fn stop(&mut self, sound: &Sound) {
        let mut sinks = self.sinks.lock().unwrap();
        if let Some(path) = &sound.path && let Some(sink) = sinks.remove(path) {
            sink.stop();
        }
    }

    fn stop_all(&mut self) {
        let mut sinks = self.sinks.lock().unwrap();
        for (_, sink) in sinks.iter() {
            sink.stop();
        }
        sinks.clear(); // Rimuove tutti i riferimenti dopo lo stop
    }

    fn is_playing(&self, sound: Option<Sound>) -> bool {
        self.clean_finished_sinks();
        let sinks = self.sinks.lock().unwrap();

        match sound {
            Some(sound) => {
                if let Some(path) = &sound.path {
                    sinks.get(path).map_or(false, |sink| !sink.empty())
                } else {
                    false
                }
            }
            None => sinks.values().any(|sink| !sink.empty()),
        }
    }

}

pub fn get_output_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.output_devices()
        .unwrap()
        .filter_map(|device| device.name().ok())
        .collect()
}

pub fn get_default_output_device() -> Option<String> {
    let host = cpal::default_host();
    host.default_output_device().unwrap().name().ok()
}

pub fn get_device_from_name(name: Option<String>) -> Option<Device> {
    let host = cpal::default_host();

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(device_name) = device.name() {
                if Some(device_name) == name {
                    return Some(device);
                }
            }
        }
    }

    host.default_output_device()
}

// Handler

pub enum AudioCommand {
    Play(Sound),
    Stop(Sound),
    StopAll,
    IsPlaying(Sender<bool>, Option<Sound>),
    SetDevice(Option<String>)
}

pub struct DesktopAudioHandler {
    sender: Sender<AudioCommand>
}

impl DesktopAudioHandler {
    pub fn new() -> Self {
        let (tx, rx): (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel();

        thread::spawn(move || {
            let mut audio = DesktopAudio::new();

            let mut last_cleanup = Instant::now();

            loop {
                let now = Instant::now();
                if now.duration_since(last_cleanup) > Duration::from_secs(3) {
                    audio.clean_finished_sinks();
                    last_cleanup = now;
                }

                let cmd = match rx.recv() {
                    Err(error) => {
                        println!("{error}");
                        continue;
                    },
                    Ok(cmd) => cmd
                };

                match cmd {
                    AudioCommand::Play(sound) => audio.play(&sound),
                    AudioCommand::Stop(sound) => audio.stop(&sound),
                    AudioCommand::StopAll => audio.stop_all(),
                    AudioCommand::IsPlaying(sender, sound) => {
                        sender
                            .send(audio.is_playing(sound))
                            .map_err(|error| format!("{error:#?}"))
                            .expect("Error occurred while sending is_playing signal:");
                    }
                    AudioCommand::SetDevice(device) => audio.set_device(device)
                }
            }
        });

        Self {  sender: tx }
    }

    pub fn set_device(&mut self, device: Option<String>) {
        let _ = self.sender.send(AudioCommand::SetDevice(device));
    }
}

impl AudioBackend for DesktopAudioHandler {
    fn play(&mut self, sound: &Sound) {
        let _ = self.sender.send(AudioCommand::Play(sound.clone()));
    }

    fn stop(&mut self, sound: &Sound) {
        let _ = self.sender.send(AudioCommand::Stop(sound.clone()));
    }

    fn stop_all(&mut self) {
        let _ = self.sender.send(AudioCommand::StopAll);
    }

    fn is_playing(&self, sound: Option<Sound>) -> bool {
        let (resp_tx, resp_rx) = mpsc::channel();

        // Manda richiesta con il canale di risposta
        if self.sender.send(AudioCommand::IsPlaying(resp_tx, sound)).is_err() {
            // thread audio chiuso o errore
            return false;
        }

        // Aspetta la risposta con un timeout ragionevole (es. 300ms)
        match resp_rx.recv() {
            Ok(is_playing) => is_playing,
            Err(_) => false, // timeout o errore -> assumiamo non in riproduzione
        }
    }
}