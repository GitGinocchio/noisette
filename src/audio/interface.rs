use crate::Sound;



pub trait AudioBackend {
    fn play(&mut self, sound : &Sound);
    fn stop(&mut self);
    fn is_playing(&self) -> bool;
}
