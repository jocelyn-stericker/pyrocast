use crate::fixed_image::FixedImage;
use crate::preferred_size::{PreferredSize, PreferredSizeExt};
use crate::vgtk_ext::*;
use pango::EllipsizeMode;
use state::{ChannelRef, StateError};
use std::sync::Arc;
use vgtk::lib::gdk_pixbuf::Pixbuf;
use vgtk::lib::gtk::{
    prelude::*, Box, FlowBox, FlowBoxChild, Label, Orientation, ScrolledWindow, SelectionMode,
    Spinner,
};
use vgtk::{gtk, Callback, Component, UpdateAction, VNode};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Props {
    pub chart_results: Option<Arc<Result<Vec<ChannelRef>, StateError>>>,
    pub selected_podcast: Option<ChannelRef>,
    pub on_select_podcast: Callback<Option<ChannelRef>>,
}

#[derive(Debug, Default, Clone)]
pub struct SearchResults {
    did_init: bool,
    pixbufs: Vec<Option<Pixbuf>>,

    props: Props,
}

impl SearchResults {
    fn render_channels(&self, channels: &[ChannelRef]) -> Vec<VNode<SearchResults>> {
        channels
            .iter()
            .map(|channel| {
                let selected = Some(channel) == self.props.selected_podcast.as_ref();

                let core = channel.core();
                let core = core.as_deref().and_then(|channel| channel.as_ref().ok());
                let label = core.map(|channel| channel.title()).unwrap_or_default();
                let small_image = core.and_then(|channel| channel.small_image());

                gtk! {
                    <FlowBoxChild
                        FlowBox::selected=selected
                        margin_bottom=10
                    >
                        <Box orientation=Orientation::Vertical>
                            <PreferredSize
                                explicit_preferred_width=130
                                explicit_preferred_height=130
                            >
                                <@FixedImage
                                    image={small_image}
                                    width=130
                                />
                            </PreferredSize>
                            <Label
                                max_width_chars=1
                                ellipsize=EllipsizeMode::End
                                hexpand=true
                                label=label
                            />
                        </Box>
                    </FlowBoxChild>
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    HandleSelectPodcast(Option<usize>),
    None,
}

impl Component for SearchResults {
    type Message = Message;
    type Properties = Props;

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::HandleSelectPodcast(podcast) => {
                let msg = podcast.and_then(|index| {
                    self.props.chart_results.as_ref().and_then(|c| {
                        c.as_ref()
                            .as_ref()
                            .ok()
                            .and_then(|c| c.get(index as usize).cloned())
                    })
                });
                self.props.on_select_podcast.send(msg);
                UpdateAction::None
            }
            Message::None => UpdateAction::None,
        }
    }

    fn create(props: Self::Properties) -> Self {
        SearchResults {
            props,
            ..Default::default()
        }
    }

    fn change(&mut self, mut props: Self::Properties) -> UpdateAction<Self> {
        std::mem::swap(&mut self.props, &mut props);
        let prev_props = props;

        if prev_props == self.props {
            UpdateAction::None
        } else {
            UpdateAction::Render
        }
    }

    fn view(&self) -> VNode<SearchResults> {
        let body = match self.props.chart_results.as_deref() {
            Option::None => gtk! { <Label label="" /> },
            Option::Some(Ok(channels)) => gtk! {
                <ScrolledWindow vexpand=true>
                    <FlowBox
                        selection_mode=SelectionMode::Browse
                        border_width=10
                        vexpand=false
                        min_children_per_line=2
                        max_children_per_line=6
                        on selected_children_changed=|flow_box| {
                            Message::HandleSelectPodcast(
                                flow_box
                                    .get_selected_children()
                                    .get(0)
                                    .map(|child| child.get_index() as usize)
                            )
                        }
                    >
                        {self.render_channels(channels)}
                    </FlowBox>
                </ScrolledWindow>
            },
            Option::Some(Err(StateError::Loading)) => gtk! {
                <Spinner
                    margin_top=10
                    on size_allocate=|s, _| {
                        s.start();
                        Message::None
                    }
                />
            },
            Option::Some(Err(StateError::DbError)) => gtk! { <Label label="Database error." /> },
            Option::Some(Err(StateError::UrlParseError(err))) => {
                gtk! { <Label label=format!("Could not parse URL: {}", &err) /> }
            }
            Option::Some(Err(StateError::NetError(err))) => {
                gtk! { <Label label=format!("Network error: {}", &err) /> }
            }
            Option::Some(Err(StateError::IoError(err))) => {
                gtk! { <Label label=format!("IO error: {}", &err) /> }
            }
            Option::Some(Err(StateError::XmlError(err))) => {
                gtk! { <Label label=format!("Network error: {}", &err) /> }
            }
        };

        gtk! {
            <Box
                orientation=Orientation::Vertical
            >
                {body}
            </Box>
        }
    }
}
