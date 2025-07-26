use crate::Sound;



pub trait AudioBackend {
    fn play(&mut self, sound : &Sound);
    fn stop(&mut self, sound : &Sound);
    fn stop_all(&mut self);
    fn is_playing(&self, sound: Option<Sound>) -> bool;
}
