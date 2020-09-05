use regex::Regex;
use serde::Deserialize;
use serde_xml_rs::from_reader;
use state::{ChannelCore, ChannelDetail, CurrentState, Episode, StateAction, StateError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssImage {
    #[serde(default)]
    pub href: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssImageLink {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RssEnclosure {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssEpisode {
    #[serde(rename = "guid")]
    /// Static identifier. Enclosure url is fallback.
    pub pk: Option<String>,

    #[serde(rename = "itunes-title")]
    /// Fallback is fallback_title.
    pub title: Option<String>,

    #[serde(rename = "title", default)]
    pub fallback_title: String,

    #[serde(rename = "link", default)]
    pub url: String,

    #[serde(rename = "pubDate", default)]
    pub date: String,

    #[serde(rename = "itunes-summary", default)]
    pub description: Option<String>,

    #[serde(rename = "description"default)]
    pub fallback_description: String,

    #[serde(rename = "itunes-duration", default)]
    pub duration: String,

    #[serde(rename = "itunes-image")]
    pub image: Option<RssImage>,

    #[serde(default)]
    pub enclosure: RssEnclosure,
}

impl RssEpisode {
    pub fn pk(&self) -> String {
        self.pk
            .clone()
            .unwrap_or_else(|| self.enclosure.url.clone())
    }

    pub fn title(&self) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| self.fallback_title.clone())
    }

    pub fn description(&self) -> String {
        self.description
            .clone()
            .unwrap_or_else(|| self.fallback_description.clone())
    }

    pub fn to_episode(&self, channel: &str, current: &CurrentState) -> Episode {
        current
            .get()
            .new_episode()
            .with_pk(self.pk())
            .with_channel(channel.to_owned())
            .with_title(self.title())
            .with_url(self.url.to_owned())
            .with_date(self.date.to_owned())
            .with_description(self.description())
            .with_duration(self.duration.clone())
            .with_image(self.image.as_ref().map(|img| img.href.clone()))
            .with_audio(self.enclosure.url.to_owned())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssChannel {
    #[serde(default)]
    pub title: String,

    #[serde(rename = "itunes-summary")]
    pub description: Option<String>,

    #[serde(rename = "description", default)]
    pub fallback_description: String,

    #[serde(default)]
    pub link: String,

    #[serde(rename = "itunes-author", default)]
    pub author: String,

    #[serde(rename = "itunes-image")]
    pub image_rss: Option<RssImage>,

    #[serde(rename = "image")]
    pub image_rss_fallback: Option<RssImageLink>,

    #[serde(rename = "item", default)]
    pub episodes: Vec<RssEpisode>,

    #[serde(default)]
    pub pk: String,

    #[serde(default)]
    pub self_url: String,
}

impl RssChannel {
    pub fn image_rss(&self) -> String {
        self.image_rss
            .as_ref()
            .map(|img| img.href.to_owned())
            .unwrap_or_else(|| {
                self.image_rss_fallback
                    .as_ref()
                    .map(|img| img.url.to_owned())
                    .unwrap_or_default()
            })
    }

    pub fn to_channel_core(&self, current: &CurrentState) -> ChannelCore {
        current
            .get()
            .new_channel_core()
            .with_pk(self.pk.to_string())
            .with_title(self.title.clone())
            .with_author(self.author.clone())
            .with_image_rss(self.image_rss())
            .with_rss(self.self_url.clone())
    }

    pub fn to_channel_detail(&self, current: &CurrentState) -> ChannelDetail {
        current
            .get()
            .new_channel_detail()
            .with_pk(self.pk.to_string())
            .with_description(
                self.description
                    .clone()
                    .unwrap_or_else(|| self.fallback_description.clone()),
            )
            .with_link(self.link.clone())
            .with_episodes(&self.episodes.iter().map(|ep| ep.pk()).collect::<Vec<_>>())
    }

    pub fn to_actions(&self, current: &CurrentState) -> Vec<StateAction> {
        let mut actions = Vec::with_capacity(self.episodes.len() + 2);

        actions.push(StateAction::SetChannelCore(
            self.pk.clone(),
            Ok(self.to_channel_core(current)),
        ));

        actions.push(StateAction::SetChannelDetail(
            self.pk.clone(),
            Ok(self.to_channel_detail(current)),
        ));

        for episode in &self.episodes {
            actions.push(StateAction::SetEpisode(
                episode.pk(),
                Ok(episode.to_episode(&self.pk, current)),
            ));
        }

        actions
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rss {
    pub channel: RssChannel,
}

impl Rss {
    pub async fn fetch(url: &str, pk: &str) -> Result<Rss, StateError> {
        let pk = pk.to_owned();
        let url = url.to_owned();

        let mut res = surf::get(&url).await?;
        let body = res.body_string().await?;

        // HACK: Namespaces are problematic, as our serde parser considers "<abc:tag>" and "<abc>"
        // to be the same. Convert "<abc:" and "</abc>" to "<abc-" and "</abc-"
        let re = Regex::new(r"(?P<tag></?\w*):").unwrap();
        let body = re.replace_all(&body, "$tag-");

        let mut feed: Rss = from_reader(body.as_bytes())?;

        // HACK: itunes_summary isn't allowed to have HTML, but that doesn't stop
        // some podcasts.
        let detag = Regex::new(r"<[^>]+>").unwrap();
        feed.channel.description = feed
            .channel
            .description
            .map(|sum| detag.replace_all(&sum, "").to_string());

        // HACK: description is allowed to have HTML. We don't deal with that yet.
        feed.channel.fallback_description = detag
            .replace_all(&feed.channel.fallback_description, "")
            .to_string();

        // HACK: same thing for episode descriptions
        for ep in &mut feed.channel.episodes {
            if let Some(description) = &mut ep.description {
                *description = detag.replace_all(description, "").to_string();
            }
            ep.fallback_description = detag.replace_all(&ep.fallback_description, "").to_string();
        }

        feed.channel.pk = pk;
        feed.channel.self_url = url;

        Ok(feed)
    }
}
