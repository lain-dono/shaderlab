use super::{Animation, Armature, Matrix};

pub enum PlayControl {
    First,
    Prev,
    PlayReverse,
    Pause,
    Play,
    Next,
    Last,
}

#[derive(Clone, Copy)]
pub enum PlayState {
    Stop,
    Play,
    PlayReverse,
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Stop
    }
}

#[derive(Default)]
pub struct Controller {
    pub current_time: u32,
    pub max_time: u32,

    pub state: PlayState,

    /// World matrices for bones.
    pub world: Vec<Matrix>,
}

impl Controller {

    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlayState::Play | PlayState::PlayReverse)
    }

    pub fn action(&mut self, control: PlayControl) {
        match control {
            PlayControl::First => self.current_time = 0,
            PlayControl::Prev if self.current_time > 0 => self.current_time -= 1,
            PlayControl::Prev => self.current_time = self.max_time,

            PlayControl::PlayReverse => self.state = PlayState::PlayReverse,
            PlayControl::Play => self.state = PlayState::Play,
            PlayControl::Pause => self.state = PlayState::Stop,

            PlayControl::Next if self.current_time < self.max_time => self.current_time += 1,
            PlayControl::Next => self.current_time = 0,
            PlayControl::Last => self.current_time = self.max_time,
        }
    }
}
