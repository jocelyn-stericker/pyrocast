use crate::fixed_image::FixedImage;
use crate::preferred_size::{PreferredSize, PreferredSizeExt};
use crate::search_detail::SearchDetail;
use crate::vgtk_ext::*;
use libhandy::{Leaflet, LeafletExt, LeafletTransitionType};
use pango::{AttrList, Attribute, EllipsizeMode, Weight};
use state::{ChannelRef, EpisodeRef, StateError};
use std::sync::Arc;
use vgtk::lib::gtk::{
    prelude::*, Align, Box as GtkBox, FlowBox, FlowBoxChild, Label, Orientation, ScrolledWindow,
    SelectionMode, Spinner,
};
use vgtk::{gtk, Callback, Component, UpdateAction, VNode};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Props {
    pub on_select_podcast: Callback<Option<ChannelRef>>,
    pub on_play: Callback<EpisodeRef>,
    pub on_subscribe: Callback<ChannelRef>,
    pub on_unsubscribe: Callback<ChannelRef>,
    pub subscriptions: Option<Arc<Result<Vec<ChannelRef>, StateError>>>,
    pub selected_podcast: Option<ChannelRef>,
    pub mobile: bool,
}

#[derive(Debug, Default, Clone)]
pub struct HomeTab {
    props: Props,
}

#[derive(Clone, Debug)]
pub enum Message {
    None,
    HandleSelectPodcast(Option<usize>),
    HandlePlay(Box<EpisodeRef>),
    HandleSubscribe(ChannelRef),
    HandleUnsubscribe(ChannelRef),
}

impl HomeTab {
    fn view_channels(&self, channels: &[ChannelRef]) -> Vec<VNode<HomeTab>> {
        channels
            .iter()
            .map(|channel| {
                let selected = Some(channel) == self.props.selected_podcast.as_ref();

                let core = channel.core();
                let core = core.as_deref().and_then(|channel| channel.as_ref().ok());
                let title = core.map(|channel| channel.title()).unwrap_or_default();
                let author = core.map(|channel| channel.author()).unwrap_or_default();
                let small_image = core.and_then(|channel| channel.small_image());

                let med_title_style = AttrList::new();
                med_title_style.insert(Attribute::new_scale(1.2).unwrap());
                med_title_style.insert(Attribute::new_weight(Weight::Bold).unwrap());

                gtk! {
                    <FlowBoxChild
                        FlowBox::selected=selected
                        valign=Align::Start
                        vexpand=false
                        hexpand=true
                        halign=Align::Fill
                        margin_bottom=10
                    >
                        <GtkBox orientation=Orientation::Horizontal>
                            <PreferredSize
                                explicit_preferred_width=50
                                explicit_preferred_height=50
                                margin_end=10
                            >
                                <@FixedImage
                                    image={small_image}
                                    width=50
                                />
                            </PreferredSize>
                            <GtkBox orientation=Orientation::Vertical>
                                <Label
                                    xalign=0.0
                                    max_width_chars=1
                                    ellipsize=EllipsizeMode::End
                                    hexpand=true
                                    halign=Align::Fill
                                    label=title
                                    attributes={Some(med_title_style.clone())}
                                />
                                <Label
                                    xalign=0.0
                                    max_width_chars=1
                                    ellipsize=EllipsizeMode::End
                                    hexpand=true
                                    halign=Align::Fill
                                    label=author
                                />
                            </GtkBox>
                        </GtkBox>
                    </FlowBoxChild>
                }
            })
            .collect()
    }

    fn view_subscription_list(&self) -> VNode<HomeTab> {
        match &self.props.subscriptions.as_deref() {
            None => gtk! { <GtkBox /> },
            Some(Err(StateError::Loading)) => {
                gtk! {
                    <Spinner
                        margin_top=10
                        on size_allocate=|s, _| {
                            s.start();
                            Message::None
                        }
                    />
                }
            }
            Some(Err(_err)) => {
                gtk! {
                    <Label label="Something has gone wrong." vexpand=true />
                }
            }
            Some(Ok(channels)) if channels.is_empty() => {
                gtk! {
                    <Label label="When you add podcasts, they will appear here." vexpand=true />
                }
            }
            Some(Ok(channels)) => {
                gtk! {
                    <ScrolledWindow vexpand=true>
                        <FlowBox
                            selection_mode=SelectionMode::Browse
                            border_width=10
                            vexpand=false
                            valign=Align::Start
                            hexpand=true
                            halign=Align::Fill
                            min_children_per_line=1
                            max_children_per_line=1
                            homogeneous=true
                            on child_activated=|flow_box, _| {
                                Message::HandleSelectPodcast(
                                    flow_box
                                        .get_selected_children()
                                        .get(0)
                                        .map(|child| child.get_index() as usize)
                                )
                            }
                            on show=|flow| {
                                // Why is this needed?
                                flow.unselect_all();
                                Message::None
                            }
                        >
                            {self.view_channels(channels)}
                        </FlowBox>
                    </ScrolledWindow>
                }
            }
        }
    }
}

impl Component for HomeTab {
    type Message = Message;
    type Properties = Props;

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::None => UpdateAction::None,
            Message::HandleSelectPodcast(podcast) => {
                let msg = podcast.and_then(|index| {
                    self.props.subscriptions.as_ref().and_then(|c| {
                        c.as_ref()
                            .as_ref()
                            .ok()
                            .and_then(|c| c.get(index as usize).cloned())
                    })
                });
                self.props.on_select_podcast.send(msg);
                UpdateAction::None
            }
            Message::HandlePlay(episode) => {
                self.props.on_play.send(*episode);
                UpdateAction::None
            }
            Message::HandleSubscribe(channel) => {
                self.props.on_subscribe.send(channel);
                UpdateAction::None
            }
            Message::HandleUnsubscribe(channel) => {
                self.props.on_unsubscribe.send(channel);
                UpdateAction::None
            }
        }
    }

    fn create(props: Self::Properties) -> Self {
        HomeTab { props }
    }

    fn change(&mut self, mut props: Self::Properties) -> UpdateAction<Self> {
        std::mem::swap(&mut self.props, &mut props);

        if props == self.props {
            UpdateAction::None
        } else {
            UpdateAction::Render
        }
    }

    fn view(&self) -> VNode<HomeTab> {
        gtk! {
            <Leaflet transition_type={LeafletTransitionType::Over} >
                <GtkBox
                    Leaflet::name="home_leaflet_browse"
                    orientation=Orientation::Vertical
                    hexpand=true
                    vexpand=true
                    property_width_request=300
                    Leaflet::is_visible_child=self.props.selected_podcast.is_none()
                >
                    {self.view_subscription_list()}
                </GtkBox>
                <GtkBox
                    Leaflet::name="home_leaflet_podcast"
                    Leaflet::is_visible_child=self.props.selected_podcast.is_some()
                >
                    {
                        if let Some(podcast) = self.props.selected_podcast.clone() {
                            let subscribed = self.props.subscriptions
                                .as_ref()
                                .and_then(|subs|
                                    subs
                                        .as_ref()
                                        .as_ref()
                                        .map(|subs| subs.contains(&podcast))
                                        .ok())
                                .unwrap_or(false);

                            gtk! {
                                <@SearchDetail
                                    podcast={Some(podcast)}
                                    mobile=self.props.mobile
                                    subscribed=subscribed
                                    on play=|episode| Message::HandlePlay(Box::new(episode))
                                    on subscribe=|channel| Message::HandleSubscribe(channel)
                                    on unsubscribe=|channel| Message::HandleUnsubscribe(channel)
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
