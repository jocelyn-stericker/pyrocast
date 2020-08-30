use crate::ItunesChannel;
use serde::Deserialize;
use state::{ChannelCore, CurrentState, StateAction, StateError};
use surf::url::Url;

#[derive(Debug, Deserialize)]
pub struct ItunesLookup {
    pub results: Vec<ItunesChannel>,
}

impl ItunesLookup {
    pub async fn fetch(id: isize) -> Result<ItunesLookup, StateError> {
        let url =
            Url::parse_with_params("https://itunes.apple.com/lookup", &[("id", id.to_string())])?;

        let mut res = surf::get(url).await?;
        Ok(res.body_json().await?)
    }

    pub fn cores(&self, current: &CurrentState) -> Vec<ChannelCore> {
        self.results
            .iter()
            .map(|item| item.to_channel_core(current))
            .collect()
    }

    pub fn to_actions(&self, current: &CurrentState) -> Vec<StateAction> {
        let mut actions = Vec::with_capacity(self.results.len());

        for core in self.cores(current) {
            actions.push(StateAction::SetChannelCore(core.pk().to_owned(), Ok(core)));
        }

        actions
    }
}
