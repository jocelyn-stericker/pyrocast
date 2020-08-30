use serde::Deserialize;
use state::StateError;
use state::{ChannelCore, CurrentState, StateAction};
use surf::url::Url;

#[derive(Debug, Deserialize)]
pub struct ItunesChartItem {
    #[serde(rename = "id", default)]
    pub pk: String,

    #[serde(rename = "name", default)]
    pub title: String,

    #[serde(rename = "artistName", default)]
    pub author: String,

    /// Actually 200x200.
    ///
    /// Note that the lookup artwork_url_100 is 100x100.
    #[serde(rename = "artworkUrl100", default)]
    pub image_200: String,
}

impl ItunesChartItem {
    pub fn to_channel_core(&self, current: &CurrentState) -> ChannelCore {
        current
            .get()
            .new_channel_core()
            .with_pk(self.pk.clone())
            .with_title(self.title.clone())
            .with_author(self.author.clone())
            .with_image_200(self.image_200.clone())
    }
}

#[derive(Debug, Deserialize)]
pub struct ItunesChartFeed {
    pub results: Vec<ItunesChartItem>,
}

#[derive(Debug, Deserialize)]
pub struct ItunesChart {
    pub feed: ItunesChartFeed,
}

impl ItunesChart {
    pub async fn fetch(
        country: &str,
        explicit: bool,
        count: usize,
    ) -> Result<ItunesChart, StateError> {
        let mut url = Url::parse("https://rss.itunes.apple.com/")?;
        url.set_path(&format!(
            "/api/v1/{}/podcasts/top-podcasts/all/{}/{}.json",
            country,
            count.min(100),
            if explicit { "explicit" } else { "non-explicit" }
        ));

        let mut res = surf::get(url).await?;
        Ok(res.body_json().await?)
    }

    fn results(&self) -> Vec<String> {
        self.feed
            .results
            .iter()
            .map(|item| item.pk.clone())
            .collect()
    }

    fn cores(&self, current: &CurrentState) -> Vec<ChannelCore> {
        self.feed
            .results
            .iter()
            .map(|item| item.to_channel_core(current))
            .collect()
    }

    pub fn to_actions(&self, current: &CurrentState) -> Vec<StateAction> {
        let mut actions = Vec::with_capacity(self.feed.results.len() + 1);
        actions.push(StateAction::SetSearchFeed {
            query: String::new(),
            results: Ok(self.results()),
        });

        for core in self.cores(current) {
            actions.push(StateAction::SetChannelCore(core.pk().to_owned(), Ok(core)));
        }

        actions
    }
}
