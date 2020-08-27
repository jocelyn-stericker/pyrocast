use crate::{CurrentState, Episode, StateError};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone)]
pub struct EpisodeRef {
    pub(crate) state: Weak<CurrentState>,
    pub(crate) pk: String,
}

impl PartialEq for EpisodeRef {
    fn eq(&self, other: &EpisodeRef) -> bool {
        self.pk == other.pk
    }
}

impl Eq for EpisodeRef {}

impl EpisodeRef {
    pub fn pk(&self) -> &str {
        &self.pk
    }

    pub fn get(&self) -> Option<Arc<Result<Episode, StateError>>> {
        let state = self.state.upgrade()?.get();
        state.episodes.get(&self.pk).map(Arc::clone)
    }
}
