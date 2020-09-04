#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Playback {
    Stopped,
    Buffering,
    Paused,
    Playing,
}

impl Playback {
    pub fn active(&self) -> bool {
        matches!(
            self,
            Playback::Buffering | Playback::Paused | Playback::Playing
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct PlayerState {
    pub episode_pk: String,
    pub channel_pk: String,
    pub playback: Playback,
    pub time: u64,
    pub duration: u64,
    pub rate: f64,
}
