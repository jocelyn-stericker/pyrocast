#[derive(Debug, Copy, Clone)]
pub enum Playback {
    Stopped,
    Buffering,
    Paused,
    Playing,
}

#[derive(Debug)]
pub struct PlayerState {
    pub episode_pk: String,
    pub channel_pk: String,
    pub playback: Playback,
    pub time: u64,
    pub duration: u64,
    pub rate: f64,
}
