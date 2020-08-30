use serde::Deserialize;
use state::{ChannelCore, CurrentState};

#[derive(Debug, Deserialize)]
pub struct ItunesChannel {
    #[serde(rename = "collectionId", default)]
    pub pk: i64,

    #[serde(rename = "trackName", default)]
    pub title: String,

    #[serde(rename = "artistName", default)]
    pub author: String,

    #[serde(rename = "artworkUrl600", default)]
    pub image_600: String,

    #[serde(rename = "feedUrl", default)]
    pub rss: String,
}

impl ItunesChannel {
    pub fn to_channel_core(&self, current: &CurrentState) -> ChannelCore {
        current
            .get()
            .new_channel_core()
            .with_pk(self.pk.to_string())
            .with_title(self.title.clone())
            .with_author(self.author.clone())
            .with_image_600(self.image_600.clone())
            .with_rss(self.rss.clone())
    }
}
