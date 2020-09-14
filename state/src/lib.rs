use crossbeam_channel::{unbounded, Sender, TryRecvError};
use futures::channel::mpsc::{channel as fchannel, Receiver as FReceiver};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::thread;

mod channel_core;
mod channel_detail;
mod channel_ref;
mod episode;
mod episode_ref;
mod image;
mod player_state;
mod state_error;

pub use channel_core::ChannelCore;
pub use channel_detail::ChannelDetail;
pub use channel_ref::ChannelRef;
pub use episode::Episode;
pub use episode_ref::EpisodeRef;
pub use image::Image;
pub use player_state::{Playback, PlayerState};
pub use state_error::StateError;

#[derive(Debug)]
pub enum StateAction {
    SetCountry(String),
    SetAllowExplicit(bool),

    SetSearchQuery(String),
    SetSearchFocus(Option<ChannelRef>),
    SetSearchFeed {
        query: String,
        results: Result<Vec<String>, StateError>,
    },

    SetHomeFocus(Option<ChannelRef>),

    SetChannelCore(String, Result<ChannelCore, StateError>),
    SetChannelDetail(String, Result<ChannelDetail, StateError>),
    SetEpisode(String, Result<Episode, StateError>),
    SetImage(String, Result<Image, StateError>),
    SetLoading(bool),
    SetPlayerState(Option<PlayerState>),
    SetSubscriptions(Result<Vec<ChannelRef>, StateError>),
}

pub(crate) type AMap<T> = Arc<HashMap<String, Arc<Result<T, StateError>>>>;

#[derive(Debug, Clone)]
/// Everything that is needed to render the UI.
pub struct State {
    pub(crate) current: Weak<CurrentState>,

    pub(crate) country: String,
    pub(crate) allow_explicit: bool,

    pub(crate) search_query: String,
    pub(crate) search_focus: Option<ChannelRef>,

    pub(crate) home_focus: Option<ChannelRef>,

    pub(crate) search_results: Arc<Result<Vec<ChannelRef>, StateError>>,

    pub(crate) channel_core: AMap<ChannelCore>,
    pub(crate) channel_detail: AMap<ChannelDetail>,
    pub(crate) episodes: AMap<Episode>,
    pub(crate) images: AMap<Image>,
    pub(crate) player_state: Arc<Option<PlayerState>>,

    pub(crate) subscriptions: Arc<Result<Vec<ChannelRef>, StateError>>,

    pub(crate) loading: bool,
}

impl State {
    pub fn country(&self) -> &str {
        &self.country
    }

    pub fn allow_explicit(&self) -> bool {
        self.allow_explicit
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn search_focus(&self) -> Option<&ChannelRef> {
        self.search_focus.as_ref()
    }

    pub fn search_results(&self) -> Arc<Result<Vec<ChannelRef>, StateError>> {
        Arc::clone(&self.search_results)
    }

    pub fn home_focus(&self) -> Option<&ChannelRef> {
        self.home_focus.as_ref()
    }

    pub fn loading(&self) -> bool {
        self.loading
    }

    pub fn new_channel_core(&self) -> ChannelCore {
        ChannelCore::default().with_current(Weak::clone(&self.current))
    }

    pub fn new_channel_detail(&self) -> ChannelDetail {
        ChannelDetail::default().with_current(Weak::clone(&self.current))
    }

    pub fn new_episode(&self) -> Episode {
        Episode::default().with_current(Weak::clone(&self.current))
    }

    pub fn player_state(&self) -> Arc<Option<PlayerState>> {
        self.player_state.clone()
    }

    pub fn channel_ref(&self, pk: String) -> ChannelRef {
        ChannelRef {
            pk,
            state: Weak::clone(&self.current),
        }
    }

    pub fn subscriptions(&self) -> Arc<Result<Vec<ChannelRef>, StateError>> {
        Arc::clone(&self.subscriptions)
    }

    pub fn playing_episode(&self) -> Option<Arc<Result<Episode, StateError>>> {
        self.player_state
            .as_ref()
            .as_ref()
            .and_then(|player_state| self.episodes.get(&player_state.episode_pk))
            .cloned()
    }

    fn references_channel(&self, channel: &str) -> bool {
        matches!(&self.search_focus, Some(search_focus) if search_focus.pk == channel)
            || matches!(&self.home_focus, Some(home_focus) if home_focus.pk == channel)
            || self
                .search_results
                .iter()
                .any(|ok| ok.iter().any(|search_item| search_item.pk == channel))
            || self.player_state.iter().any(|ok| ok.channel_pk == channel)
    }

    fn references_image(&self, image: &str) -> bool {
        self.channel_core.iter().any(|channel| {
            channel
                .1
                .iter()
                .any(|channel| channel.references_image(image))
        }) || self.episodes.iter().any(|episode| {
            episode
                .1
                .iter()
                .any(|episode| episode.references_image(image))
        })
    }

    pub fn new() -> Self {
        State {
            current: Weak::new(),
            country: String::from("CA"),
            allow_explicit: false,
            search_query: String::new(),
            search_focus: None,
            search_results: Arc::new(Result::Err(StateError::Loading)),
            home_focus: None,
            channel_core: Default::default(),
            channel_detail: Default::default(),
            episodes: Default::default(),
            images: Default::default(),
            loading: true,
            player_state: Arc::new(Option::None),
            subscriptions: Arc::new(Result::Err(StateError::Loading)),
        }
    }

    fn with_current(&self, current: Weak<CurrentState>) -> State {
        let mut next = self.clone();
        next.current = current;

        next
    }

    fn apply(&self, actions: Vec<StateAction>) -> State {
        let mut next = self.clone();
        let mut next_channel_core = None;
        let mut next_channel_detail = None;
        let mut next_episodes = None;
        let mut next_images = None;

        for action in actions {
            match action {
                StateAction::SetCountry(country) => {
                    next.country = country;
                }
                StateAction::SetAllowExplicit(allow_explicit) => {
                    next.allow_explicit = allow_explicit;
                }
                StateAction::SetSearchQuery(query) => {
                    if next.search_query != query {
                        next.search_query = query;
                        next.search_results = Arc::new(Result::Err(StateError::Loading));
                    }
                }
                StateAction::SetSearchFocus(focus) => {
                    next.search_focus = focus;
                }
                StateAction::SetSearchFeed { query, results } => {
                    if next.search_query == query {
                        next.search_results = Arc::new(results.map(|results| {
                            results
                                .into_iter()
                                .map(|pk| ChannelRef {
                                    pk,
                                    state: Weak::clone(&self.current),
                                })
                                .collect()
                        }));
                    }
                }
                StateAction::SetHomeFocus(focus) => {
                    next.home_focus = focus;
                }

                StateAction::SetChannelCore(pk, mut core) => {
                    if next.references_channel(&pk) {
                        if let Ok(core) = &mut core {
                            if let Some(curr_core) = self.channel_core.get(&pk) {
                                if let Ok(curr_core) = curr_core.as_ref() {
                                    core.update(curr_core);
                                }
                            }
                        }

                        next_channel_core
                            .get_or_insert_with(|| (*self.channel_core).clone())
                            .insert(pk, Arc::new(core));
                    }
                }
                StateAction::SetChannelDetail(pk, detail) => {
                    if next.references_channel(&pk) {
                        next_channel_detail
                            .get_or_insert_with(|| (*self.channel_detail).clone())
                            .insert(pk, Arc::new(detail));
                    }
                }
                StateAction::SetEpisode(pk, episode) => {
                    next_episodes
                        .get_or_insert_with(|| (*self.episodes).clone())
                        .insert(pk, Arc::new(episode));
                }
                StateAction::SetImage(pk, image) => {
                    if next.references_image(&pk) {
                        next_images
                            .get_or_insert_with(|| (*self.images).clone())
                            .insert(pk, Arc::new(image));
                    }
                }
                StateAction::SetLoading(loading) => {
                    next.loading = loading;
                }
                StateAction::SetPlayerState(player_state) => {
                    next.player_state = Arc::new(player_state);
                }
                StateAction::SetSubscriptions(subscriptions) => {
                    next.subscriptions = Arc::new(subscriptions);
                }
            }
        }

        if let Some(channel_core) = next_channel_core {
            next.channel_core = Arc::new(channel_core);
        }
        if let Some(channel_detail) = next_channel_detail {
            next.channel_detail = Arc::new(channel_detail);
        }
        if let Some(episodes) = next_episodes {
            next.episodes = Arc::new(episodes);
        }
        if let Some(images) = next_images {
            next.images = Arc::new(images);
        }

        next
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

pub struct CurrentState(Arc<RwLock<Arc<State>>>, Sender<Vec<StateAction>>);

impl std::fmt::Debug for CurrentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CurrentState")
    }
}

impl CurrentState {
    /// Creates a state that can be gotten or updated (async).
    ///
    /// Also creates a notifier that can be used to figure out when the state has been updated.
    pub fn new() -> (Arc<CurrentState>, FReceiver<()>) {
        let state = Arc::new(RwLock::new(Arc::new(State::new())));
        let (send_action, receive_action) = unbounded();
        let (mut send_update, receive_update) = fchannel(1);

        let current_state = Arc::new(CurrentState(state.clone(), send_action));
        let with_current = Arc::new(
            state
                .read()
                .unwrap()
                .with_current(Arc::downgrade(&current_state)),
        );
        *state.write().unwrap() = with_current;

        thread::spawn(move || {
            let send_actions = |actions: Vec<StateAction>| {
                let curr = state.read().unwrap().clone();
                let next = Arc::new(curr.apply(actions));
                *state.write().unwrap() = next;
            };

            loop {
                match receive_action.try_recv() {
                    Ok(actions) => {
                        send_actions(actions);
                    }
                    Err(TryRecvError::Empty) => {
                        // Only send if they're not already waiting for an update.
                        let _ = send_update.try_send(());
                        if let Ok(actions) = receive_action.recv() {
                            send_actions(actions);
                        } else {
                            break;
                        }
                    }
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        (current_state, receive_update)
    }

    pub fn get(&self) -> Arc<State> {
        self.0.read().unwrap().clone()
    }

    pub fn update(&self, actions: Vec<StateAction>) {
        self.1.send(actions).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let (current_state, mut wait_for_update) = CurrentState::new();

        assert_eq!(current_state.get().search_query(), "");
        assert_eq!(wait_for_update.try_next().is_err(), true);

        current_state.update(vec![StateAction::SetSearchQuery(String::from(
            "This American Life",
        ))]);
        assert_eq!(current_state.get().search_query(), "");

        while wait_for_update.try_next().is_err() {}
        assert_eq!(current_state.get().search_query(), "This American Life");
        assert_eq!(
            current_state.get().search_results().as_ref(),
            &Err(StateError::Loading)
        );

        current_state.update(vec![
            StateAction::SetSearchFeed {
                query: String::from("This American Life"),
                results: Ok(vec![String::from("tal")]),
            },
            current_state
                .get()
                .new_channel_core()
                .with_pk(String::from("tal"))
                .with_title(String::from("This American Life"))
                .build(),
            current_state
                .get()
                .new_channel_core()
                .with_pk(String::from("invalid"))
                .with_title(String::from("Not present"))
                .build(),
        ]);
        while wait_for_update.try_next().is_err() {}

        let state = current_state.get();
        let search = state.search_results();
        let search = Result::as_ref(&search).unwrap();
        let core = search.get(0).unwrap().core();
        let core = core.as_deref();
        assert_eq!(
            core.unwrap().as_ref().unwrap().title(),
            "This American Life"
        );
        assert!(current_state.get().channel_core.get("invalid").is_none());
    }
}
