use crate::{ChannelCore, CurrentState, Episode, EpisodeRef, StateAction, StateError};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, Default)]
pub struct ChannelDetail {
    state: Weak<CurrentState>,

    pk: String,

    description: String,
    episodes: Vec<EpisodeRef>,
}

impl ChannelDetail {
    pub fn pk(&self) -> &str {
        &self.pk
    }

    pub fn with_pk(mut self, pk: String) -> Self {
        self.pk = pk;
        self
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn episodes(&self) -> &[EpisodeRef] {
        &self.episodes
    }

    pub fn with_episodes(mut self, episodes: &[Episode]) -> Self {
        self.episodes = episodes
            .iter()
            .map(|episode| EpisodeRef {
                state: Weak::clone(&self.state),
                pk: String::from(episode.pk()),
            })
            .collect();

        self
    }

    pub fn build(self) -> StateAction {
        StateAction::SetChannelDetail(String::from(&self.pk), Ok(self))
    }

    pub fn core(&self) -> Option<Arc<Result<ChannelCore, StateError>>> {
        let state = self.state.upgrade()?.get();

        state.channel_core.get(&self.pk).map(Arc::clone)
    }

    pub(crate) fn with_current(mut self, current: Weak<CurrentState>) -> Self {
        self.state = current;
        self
    }
}
