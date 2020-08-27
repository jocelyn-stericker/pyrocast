use crate::{ChannelDetail, CurrentState, Image, StateAction, StateError};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, Default)]
pub struct ChannelCore {
    state: Weak<CurrentState>,

    /// Generally the iTunes ID. Can also be the RSS URL.
    pk: String,

    title: String,
    author: String,

    image_200: Option<String>,
    image_600: Option<String>,
    image_rss: Option<String>,

    rss: Option<String>,
}

impl ChannelCore {
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

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_image_200(mut self, image_200: String) -> Self {
        self.image_200 = Some(image_200);
        self
    }

    pub fn with_image_600(mut self, image_600: String) -> Self {
        self.image_600 = Some(image_600);
        self
    }

    pub fn with_image_rss(mut self, image_rss: String) -> Self {
        self.image_rss = Some(image_rss);
        self
    }

    pub fn small_image(&self) -> Option<Arc<Result<Image, StateError>>> {
        let state = self.state.upgrade()?.get();

        let mut fallback = None;

        if let Some(image_200) = self
            .image_200
            .as_ref()
            .and_then(|url| state.images.get(url))
        {
            if image_200.is_err() {
                fallback = Some(Arc::clone(image_200));
            } else {
                return Some(Arc::clone(image_200));
            }
        }

        if let Some(image_600) = self
            .image_600
            .as_ref()
            .and_then(|url| state.images.get(url))
        {
            return Some(Arc::clone(image_600));
        }

        fallback
    }

    pub fn big_image(&self) -> Option<Arc<Result<Image, StateError>>> {
        let state = self.state.upgrade()?.get();

        let mut fallback = None;

        if let Some(image_600) = self
            .image_600
            .as_ref()
            .and_then(|url| state.images.get(url))
        {
            if image_600.is_err() {
                fallback = Some(Arc::clone(image_600));
            } else {
                return Some(Arc::clone(image_600));
            }
        }

        if let Some(image_rss) = self
            .image_rss
            .as_ref()
            .and_then(|url| state.images.get(url))
        {
            return Some(Arc::clone(image_rss));
        }

        fallback
    }

    pub fn details(&self) -> Option<Arc<Result<ChannelDetail, StateError>>> {
        let state = self.state.upgrade()?.get();

        state.channel_detail.get(&self.pk).map(Arc::clone)
    }

    pub fn build(self) -> StateAction {
        StateAction::SetChannelCore(String::from(&self.pk), Ok(self))
    }

    pub(crate) fn with_current(mut self, current: Weak<CurrentState>) -> Self {
        self.state = current;
        self
    }

    pub(crate) fn update(&mut self, other: &ChannelCore) {
        if self.image_200.is_none() {
            self.image_200 = other.image_200.clone();
        }
        if self.image_600.is_none() {
            self.image_600 = other.image_600.clone();
        }
        if self.image_rss.is_none() {
            self.image_rss = other.image_rss.clone();
        }
        if self.rss.is_none() {
            self.rss = other.rss.clone();
        }
    }

    pub(crate) fn references_image(&self, image: &str) -> bool {
        matches!(&self.image_200, Some(image_200) if image_200 == image)
            || matches!(&self.image_600, Some(image_600) if image_600 == image)
            || matches!(&self.image_rss, Some(image_rss) if image_rss == image)
    }
}
