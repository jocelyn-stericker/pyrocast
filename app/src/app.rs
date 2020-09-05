use crate::home_tab::HomeTab;
use crate::now_playing::NowPlaying;
use crate::search_tab::SearchTab;
use crate::settings_tab::SettingsTab;
use crate::vgtk_ext::*;
use libhandy::{
    CenteringPolicy, HeaderBar, HeaderBarExt, Squeezer, SqueezerExt, ViewSwitcher, ViewSwitcherBar,
    ViewSwitcherBarExt, ViewSwitcherExt, ViewSwitcherPolicy,
};
use loader::{Loader, Query};
use player::PlayerAction;
use state::{ChannelRef, CurrentState, EpisodeRef, Image as ImageObj, State, StateAction};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use vgtk::lib::gio::ApplicationFlags;
use vgtk::lib::gtk::{
    prelude::*, Align, Application, Box as GtkBox, Button, Image, Label, Orientation, Stack,
    WidgetExt, Window,
};
use vgtk::{ext::*, gtk, Component, UpdateAction, VNode};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Tab {
    NowPlaying,
    Home,
    Search,
    Settings,
}

#[derive(Clone, Debug)]
pub enum Message {
    None,
    Quit,
    SetMobile(bool),
    SetTab(Tab),
    SetSearchDetail(Option<ChannelRef>),
    SetSearchQuery(String),

    // Playback
    HandlePlay(EpisodeRef),
    HandleSkipBack,
    HandleSkipForward,
    HandlePause,
    HandleUnpause,
    HandleSeek(u64),
    HandleRateChange(f64),

    // External
    InitDispatch(Arc<CurrentState>),
    InitLoaderWIP(Loader, Sender<PlayerAction>),
    StateChanged(Arc<State>),
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Search
    }
}

#[derive(Default, Clone)]
pub struct App {
    mobile: bool,
    tab: Tab,
    state: Arc<State>,
    current: Option<Arc<CurrentState>>,
    loader: Option<Loader>,
    player: Option<Sender<PlayerAction>>,
}

impl App {}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::None => UpdateAction::None,
            Message::Quit => {
                eprintln!("exit");
                vgtk::quit();
                UpdateAction::None
            }
            Message::SetMobile(mobile) => {
                self.mobile = mobile;
                UpdateAction::Render
            }
            Message::SetTab(tab) => {
                if self.tab != tab {
                    self.tab = tab;
                    UpdateAction::Render
                } else {
                    UpdateAction::None
                }
            }
            Message::SetSearchDetail(search) => {
                if let Some(current) = &self.current {
                    current.update(vec![StateAction::SetSearchFocus(search.clone())]);
                }
                if let Some(loader) = &self.loader {
                    if let Some(search) = search {
                        loader.queue(Query::ItunesLookup {
                            pk: search.pk().to_owned(),
                        });
                    }
                }
                UpdateAction::None
            }
            Message::SetSearchQuery(search) => {
                if let Some(current) = &self.current {
                    current.update(vec![StateAction::SetSearchQuery(search.clone())]);
                }
                if let Some(loader) = &self.loader {
                    if search == "" {
                        loader.queue(Query::ItunesChart);
                    } else {
                        loader.queue(Query::ItunesSearch { query: search });
                    }
                }
                UpdateAction::None
            }

            // Playback
            Message::HandlePlay(episode) => {
                if let (Some(player), Some(Ok(episode))) = (&self.player, episode.get().as_deref())
                {
                    player
                        .send(PlayerAction::PlayRemote {
                            episode_pk: episode.pk().to_owned(),
                            channel_pk: episode.channel().pk().to_owned(),
                            uri: episode.audio().to_owned(),
                        })
                        .unwrap();

                    if let (Some(image), Some(loader)) = (&episode.image(), &self.loader) {
                        if let Ok(image) = image.as_ref() {
                            if !image.loaded() {
                                loader.queue(Query::Image {
                                    image: Arc::new(ImageObj::new(&image.pk)),
                                    associated_query: None,
                                });
                            }
                        }
                    }
                }

                UpdateAction::None
            }
            Message::HandleSkipBack => {
                let time = self
                    .state
                    .player_state()
                    .as_ref()
                    .as_ref()
                    .map(|state| state.time)
                    .unwrap_or_default() as i64;

                if let Some(player) = &self.player {
                    player
                        .send(PlayerAction::SetTime((time - 15000).max(0) as u64))
                        .unwrap();
                }
                UpdateAction::None
            }
            Message::HandleSkipForward => {
                let time = self
                    .state
                    .player_state()
                    .as_ref()
                    .as_ref()
                    .map(|state| state.time)
                    .unwrap_or_default();

                if let Some(player) = &self.player {
                    player.send(PlayerAction::SetTime(time + 30000)).unwrap();
                }
                UpdateAction::None
            }
            Message::HandlePause => {
                if let Some(player) = &self.player {
                    player.send(PlayerAction::Pause).unwrap();
                }
                UpdateAction::None
            }
            Message::HandleUnpause => {
                if let Some(player) = &self.player {
                    player.send(PlayerAction::Unpause).unwrap();
                }
                UpdateAction::None
            }
            Message::HandleSeek(t) => {
                if let Some(player) = &self.player {
                    player.send(PlayerAction::SetTime(t)).unwrap();
                }
                UpdateAction::None
            }
            Message::HandleRateChange(rate) => {
                if let Some(player) = &self.player {
                    player.send(PlayerAction::SetRate(rate)).unwrap();
                }
                UpdateAction::None
            }

            // External
            Message::InitDispatch(current) => {
                self.current = Some(current);
                UpdateAction::None
            }
            Message::InitLoaderWIP(loader, player) => {
                self.loader = Some(loader);
                self.player = Some(player);
                UpdateAction::None
            }
            Message::StateChanged(state) => {
                self.state = state;
                UpdateAction::Render
            }
        }
    }

    fn view(&self) -> VNode<App> {
        let tab = self.tab;

        gtk! {
            <Application::new_unwrap(Some("ca.nettek.pyrocast"), ApplicationFlags::empty())>
                <Window
                    on destroy=|_| Message::Quit
                    on show=|win| {
                        // Setup non parent-child relationships.
                        let named_objects = win.get_named_descendants();
                        let stack: &Stack = named_objects.get_downcast("pyrocast_app_stack").unwrap();
                        let view_switcher: &ViewSwitcher = named_objects
                            .get_downcast("pyrocast_app_view_switcher")
                            .unwrap();
                        let view_switcher_bar: &ViewSwitcherBar = named_objects
                            .get_downcast("pyrocast_app_view_switcher_bar")
                            .unwrap();
                        view_switcher.set_stack(Some(stack));
                        view_switcher_bar.set_stack(Some(stack));

                        let header_bar: &HeaderBar = named_objects
                            .get_downcast("pyrocast_app_header_bar")
                            .unwrap();
                        let title: &Squeezer = named_objects.get_downcast("pyrocast_app_title").unwrap();

                        header_bar.remove(title);
                        header_bar.set_custom_title(Some(title));
                        Message::None
                    }
                    title="Pyrocast"
                    default_width=600
                    default_height=800
                >
                    <HeaderBar
                        show_close_button=true
                        centering_policy=CenteringPolicy::Strict
                        widget_name="pyrocast_app_header_bar"
                    >
                        <Button
                            visible=self.state.search_focus().is_some() && tab == Tab::Search
                            valign=Align::Center
                            on clicked=|_| Message::SetSearchDetail(None)
                        >
                            <Image
                                property_icon_name="go-previous-symbolic"
                                property_icon_size=1
                            />
                        </Button>
                        <Squeezer
                            widget_name="pyrocast_app_title"
                            on property_visible_child_notify=|switcher|
                                Message::SetMobile(
                                    !switcher.get_visible_child().map(|c| c.is::<ViewSwitcher>()).unwrap_or(false)
                                )
                        >
                            <ViewSwitcher
                                policy=ViewSwitcherPolicy::Wide
                                widget_name="pyrocast_app_view_switcher"
                            />
                            <Label label="Pyrocast" />
                        </Squeezer>
                    </HeaderBar>
                    <GtkBox orientation=Orientation::Vertical>
                        <Stack
                            vexpand=true
                            widget_name="pyrocast_app_stack"
                            on property_visible_child_notify=|stack| {
                                let tab_name = stack.get_visible_child_name().map(|s| s.to_string()).unwrap_or_default();
                                let new_tab = match tab_name.as_str() {
                                    "pyrocast_tab_playing" => Tab::NowPlaying,
                                    "pyrocast_tab_home" => Tab::Home,
                                    "pyrocast_tab_search" => Tab::Search,
                                    "pyrocast_tab_settings" => Tab::Settings,
                                    _ => tab
                                };

                                Message::SetTab(new_tab)
                            }
                        >
                            <GtkBox
                                Stack::title="Playing"
                                Stack::icon_name="media-playback-start-symbolic"
                                Stack::selected=tab == Tab::NowPlaying
                                Stack::name="pyrocast_tab_playing"
                            >
                                <@NowPlaying
                                    player_state=self.state.player_state()
                                    episode_info=self.state.playing_episode()
                                    on skip_back=|_| Message::HandleSkipBack
                                    on skip_forward=|_| Message::HandleSkipForward
                                    on pause=|_| Message::HandlePause
                                    on unpause=|_| Message::HandleUnpause
                                    on seek=|t| Message::HandleSeek(t)
                                    on rate_change=|rate| Message::HandleRateChange(rate)
                                />
                            </GtkBox>
                            <GtkBox
                                Stack::title="Home"
                                Stack::icon_name="user-home-symbolic"
                                Stack::selected=tab == Tab::Home
                                Stack::name="pyrocast_tab_home"
                            >
                                <@HomeTab />
                            </GtkBox>
                            <GtkBox
                                Stack::title="Search"
                                Stack::icon_name="system-search-symbolic"
                                Stack::selected=tab == Tab::Search
                                Stack::name="pyrocast_tab_search"
                            >
                                <@SearchTab
                                    chart_results=Some(self.state.search_results())
                                    selected_podcast=self.state.search_focus().cloned()
                                    mobile=self.mobile
                                    on select_podcast=|podcast| Message::SetSearchDetail(podcast)
                                    on play=|episode| Message::HandlePlay(episode)
                                    on search=|search| Message::SetSearchQuery(search)
                                />
                            </GtkBox>
                            <GtkBox
                                Stack::title="Settings"
                                Stack::icon_name="preferences-system-symbolic"
                                Stack::selected=tab == Tab::Settings
                                Stack::name="pyrocast_tab_settings"
                            >
                                <@SettingsTab />
                            </GtkBox>
                        </Stack>
                        <ViewSwitcherBar
                            visible=self.mobile
                            reveal=self.mobile
                            widget_name="pyrocast_app_view_switcher_bar"
                        />
                    </GtkBox>
                </Window>
            </Application>
        }
    }
}
