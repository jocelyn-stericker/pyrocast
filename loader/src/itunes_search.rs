use crate::ItunesChannel;
use serde::Deserialize;
use state::{ChannelCore, CurrentState, StateAction, StateError};
use surf::url::Url;

#[derive(Debug, Deserialize)]
pub struct ItunesSearch {
    pub results: Vec<ItunesChannel>,

    #[serde(default)]
    pub search_query: String,
}

impl ItunesSearch {
    pub async fn fetch(
        country: &str,
        explicit: bool,
        count: usize,
        query: &str,
    ) -> Result<ItunesSearch, StateError> {
        let mut url = Url::parse("https://itunes.apple.com/search")?;
        url.query_pairs_mut()
            .append_pair("term", query)
            .append_pair("country", country)
            .append_pair("count", &count.min(200).to_string())
            .append_pair("media", "podcast")
            .append_pair("explicit", if explicit { "Yes" } else { "No" });

        let mut res = surf::get(url).await?;

        let mut wrapper: ItunesSearch = res.body_json().await?;
        wrapper.results = wrapper
            .results
            .into_iter()
            .filter(|res| res.rss != "")
            .collect();

        wrapper.search_query = query.to_owned();

        Ok(wrapper)
    }

    pub fn results(&self) -> Vec<String> {
        self.results
            .iter()
            .map(|item| item.pk.to_string())
            .collect()
    }

    pub fn cores(&self, current: &CurrentState) -> Vec<ChannelCore> {
        self.results
            .iter()
            .map(|item| item.to_channel_core(current))
            .collect()
    }

    pub fn to_actions(&self, current: &CurrentState) -> Vec<StateAction> {
        let mut actions = Vec::with_capacity(self.results.len() + 1);
        actions.push(StateAction::SetSearchFeed {
            query: self.search_query.clone(),
            results: Ok(self.results()),
        });

        for core in self.cores(current) {
            actions.push(StateAction::SetChannelCore(core.pk().to_owned(), Ok(core)));
        }

        actions
    }
}
