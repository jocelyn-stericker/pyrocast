use crate::search_detail::SearchDetail;
use crate::search_results::SearchResults;
use crate::vgtk_ext::*;
use libhandy::{
    Column, ColumnExt, Leaflet, LeafletExt, LeafletTransitionType, SearchBar, SearchBarExt,
};
use state::{ChannelRef, EpisodeRef, StateError};
use std::sync::Arc;
use vgtk::lib::gtk::{prelude::*, Box as GtkBox, Label, Orientation, SearchEntry};
use vgtk::{gtk, Callback, Component, UpdateAction, VNode};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Props {
    pub on_select_podcast: Callback<Option<ChannelRef>>,
    pub on_play: Callback<EpisodeRef>,
    pub on_search: Callback<String>,
    pub selected_podcast: Option<ChannelRef>,
    pub chart_results: Option<Arc<Result<Vec<ChannelRef>, StateError>>>,
    pub mobile: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SearchTab {
    props: Props,
}

#[derive(Clone, Debug)]
pub enum Message {
    HandleSelectPodcast(Option<ChannelRef>),
    HandlePlay(Box<EpisodeRef>),
    HandleSearch(String),
}

impl Component for SearchTab {
    type Message = Message;
    type Properties = Props;

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::HandleSelectPodcast(podcast) => {
                self.props.on_select_podcast.send(podcast);
                UpdateAction::None
            }
            Message::HandlePlay(episode) => {
                self.props.on_play.send(*episode);
                UpdateAction::None
            }
            Message::HandleSearch(search) => {
                self.props.on_search.send(search);
                UpdateAction::None
            }
        }
    }

    fn create(props: Self::Properties) -> Self {
        SearchTab { props }
    }

    fn change(&mut self, mut props: Self::Properties) -> UpdateAction<Self> {
        std::mem::swap(&mut self.props, &mut props);

        if props == self.props {
            UpdateAction::None
        } else {
            UpdateAction::Render
        }
    }

    fn view(&self) -> VNode<SearchTab> {
        gtk! {
            <Leaflet transition_type={LeafletTransitionType::Over} >
                <GtkBox
                    Leaflet::name="search_leaflet_browse"
                    Leaflet::is_visible_child=self.props.selected_podcast.is_none()
                    orientation=Orientation::Vertical
                    property_width_request=300
                    hexpand=false
                >
                    <SearchBar search_mode=true show_close_button=false>
                        <Column hexpand=true maximum_width=600>
                            <SearchEntry
                                on property_text_notify=|entry| Message::HandleSearch(entry.get_text().to_string())
                            />
                        </Column>
                    </SearchBar>
                    <@SearchResults
                        chart_results=self.props.chart_results.clone()
                        selected_podcast=self.props.selected_podcast.clone()
                        on select_podcast=|podcast| Message::HandleSelectPodcast(podcast)
                    />
                </GtkBox>
                <GtkBox
                    Leaflet::name="search_leaflet_podcast"
                    Leaflet::is_visible_child=self.props.selected_podcast.is_some()
                >
                    {
                        if let Some(podcast) = self.props.selected_podcast.clone() {
                            gtk! {
                                <@SearchDetail
                                    podcast={Some(podcast)}
                                    mobile=self.props.mobile
                                    on play=|episode| Message::HandlePlay(Box::new(episode))
                                />
                            }
                        } else {
                            gtk! { <Label
                                label="Select a podcast."
                                property_width_request=300
                                hexpand=true
                            /> }
                        }
                    }
                </GtkBox>
            </Leaflet>
        }
    }
}
