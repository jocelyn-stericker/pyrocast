use crate::{ChannelCore, ChannelDetail, CurrentState, StateError};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone)]
pub struct ChannelRef {
    pub(crate) state: Weak<CurrentState>,
    pub(crate) pk: String,
}

impl PartialEq for ChannelRef {
    fn eq(&self, other: &ChannelRef) -> bool {
        self.pk == other.pk
    }
}

impl Eq for ChannelRef {}

impl ChannelRef {
    pub fn pk(&self) -> &str {
        &self.pk
    }

    pub fn core(&self) -> Option<Arc<Result<ChannelCore, StateError>>> {
        let state = self.state.upgrade()?.get();
        state.channel_core.get(&self.pk).map(Arc::clone)
    }

    pub fn details(&self) -> Option<Arc<Result<ChannelDetail, StateError>>> {
        let state = self.state.upgrade()?.get();
        state.channel_detail.get(&self.pk).map(Arc::clone)
    }
}
