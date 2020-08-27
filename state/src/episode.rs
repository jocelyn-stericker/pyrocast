use crate::{ChannelRef, CurrentState, Image, StateAction, StateError};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, Default)]
pub struct Episode {
    state: Weak<CurrentState>,

    pk: String,
    channel: String,

    title: String,
    url: String,
    date: String,
    description: String,
    duration: String,

    image: Option<String>,
    audio: String,
}

impl Episode {
    pub fn pk(&self) -> &str {
        &self.pk
    }

    pub fn with_pk(mut self, pk: String) -> Self {
        self.pk = pk;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn with_date(mut self, date: String) -> Self {
        self.date = date;
        self
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn duration(&self) -> &str {
        &self.duration
    }

    pub fn with_duration(mut self, duration: String) -> Self {
        self.duration = duration;
        self
    }

    pub fn audio(&self) -> &str {
        &self.audio
    }

    pub fn with_audio(mut self, audio: String) -> Self {
        self.audio = audio;
        self
    }

    pub fn channel(&self) -> ChannelRef {
        ChannelRef {
            pk: self.pk.clone(),
            state: self.state.clone(),
        }
    }

    pub fn image(&self) -> Option<Arc<Result<Image, StateError>>> {
        let state = self.state.upgrade()?.get();

        let mut fallback = None;

        if let Some(image) = self.image.as_ref().and_then(|url| state.images.get(url)) {
            if image.is_err() {
                fallback = Some(Arc::clone(image));
            } else {
                return Some(Arc::clone(image));
            }
        }

        if let Some(core) = self.channel().core() {
            if let Ok(core) = core.as_ref() {
                if let Some(channel_image) = core.big_image() {
                    return Some(Arc::clone(&channel_image));
                }
            }
        }

        fallback
    }

    pub fn with_image(mut self, image: String) -> Self {
        self.image = Some(image);
        self
    }

    pub fn build(self) -> StateAction {
        StateAction::SetEpisode(String::from(&self.pk), Ok(self))
    }

    pub(crate) fn with_current(mut self, current: Weak<CurrentState>) -> Self {
        self.state = current;
        self
    }

    pub(crate) fn references_image(&self, image: &str) -> bool {
        matches!(&self.image, Some(ep_image) if ep_image == image)
    }
}
