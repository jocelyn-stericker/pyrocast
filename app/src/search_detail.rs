use crate::fixed_image::FixedImage;
use chrono::{naive::NaiveTime, DateTime};
use libhandy::{Column, ColumnExt, Squeezer, SqueezerExt};
use pango::EllipsizeMode;
use pango::{AttrList, Attribute, Variant, Weight};
use state::{ChannelCore, ChannelDetail, ChannelRef, EpisodeRef, StateError};
use std::sync::Arc;
use vgtk::lib::gtk::{
    prelude::*, Align, Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, SelectionMode, Viewport,
};
use vgtk::{ext::*, gtk, Callback, Component, UpdateAction, VNode};
use xml::escape::{escape_str_attribute, escape_str_pcdata};

#[derive(Debug, Default, Clone)]
pub struct Props {
    pub podcast: Option<ChannelRef>,
    pub on_play: Callback<EpisodeRef>,
    pub mobile: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SearchDetail {
    props: Props,
    episode_limit: usize,

    prev_core: Option<Arc<Result<ChannelCore, StateError>>>,
    prev_detail: Option<Arc<Result<ChannelDetail, StateError>>>,
}

#[derive(Clone, Debug)]
pub enum Message {
    HandlePlay(usize),
    HandleShowMore,
}

impl Component for SearchDetail {
    type Message = Message;
    type Properties = Props;

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::HandlePlay(idx) => {
                if let Some(channel) = &self.props.podcast {
                    let detail = channel.details();
                    let detail = detail.as_deref().and_then(|channel| channel.as_ref().ok());
                    let episodes = detail.map(|channel| channel.episodes()).unwrap_or_default();

                    if let Some(episode) = episodes.get(idx) {
                        self.props.on_play.send(episode.clone());
                    }
                }
                UpdateAction::None
            }
            Message::HandleShowMore => {
                self.episode_limit += 10;
                UpdateAction::Render
            }
        }
    }

    fn create(props: Self::Properties) -> Self {
        SearchDetail {
            props,
            episode_limit: 20,
            prev_core: None,
            prev_detail: None,
        }
    }

    fn change(&mut self, props: Self::Properties) -> UpdateAction<Self> {
        // Do we need to rerender?
        // TODO: this is super ugly! How can we make this better?
        if props.podcast != self.props.podcast {
            self.episode_limit = 20;
            self.prev_core = props.podcast.as_ref().and_then(|pod| pod.core());
            self.prev_detail = props.podcast.as_ref().and_then(|pod| pod.details());
            self.props = props;
            UpdateAction::Render
        } else {
            let next_core = props.podcast.as_ref().and_then(|pod| pod.core());
            let next_detail = props.podcast.as_ref().and_then(|pod| pod.details());
            let mut changed = false;

            if let (Some(prev_core), Some(next_core)) = (&self.prev_core, &next_core) {
                if !Arc::ptr_eq(prev_core, next_core) {
                    changed = true;
                }
            } else if self.prev_core.is_some() != next_core.is_some() {
                changed = true;
            }

            if let (Some(prev_detail), Some(next_detail)) = (&self.prev_detail, &next_detail) {
                if !Arc::ptr_eq(prev_detail, next_detail) {
                    changed = true
                }
            } else if self.prev_detail.is_some() != next_detail.is_some() {
                changed = true
            }

            if changed {
                self.prev_core = props.podcast.as_ref().and_then(|pod| pod.core());
                self.prev_detail = props.podcast.as_ref().and_then(|pod| pod.details());
                self.props = props;
                return UpdateAction::Render;
            }

            if props.mobile != self.props.mobile {
                self.props = props;
                UpdateAction::Render
            } else {
                UpdateAction::None
            }
        }
    }

    fn view(&self) -> VNode<SearchDetail> {
        let contents = if let Some(channel) = &self.props.podcast {
            let desk_title_style = AttrList::new();
            desk_title_style.insert(Attribute::new_scale(1.5).unwrap());
            desk_title_style.insert(Attribute::new_weight(Weight::Bold).unwrap());

            let med_title_style = AttrList::new();
            med_title_style.insert(Attribute::new_scale(1.2).unwrap());
            med_title_style.insert(Attribute::new_weight(Weight::Bold).unwrap());

            let episode_title_style = AttrList::new();
            episode_title_style.insert(Attribute::new_weight(Weight::Bold).unwrap());

            let episode_meta_style = AttrList::new();
            episode_meta_style.insert(Attribute::new_scale(0.8).unwrap());
            episode_meta_style.insert(Attribute::new_variant(Variant::SmallCaps).unwrap());

            let core = channel.core();
            let core = core.as_deref().and_then(|channel| channel.as_ref().ok());
            let title = core.map(|channel| channel.title()).unwrap_or_default();
            let author = core.map(|channel| channel.author()).unwrap_or_default();
            let big_image = core.and_then(|channel| channel.big_image());

            let details = channel.details();
            let details = details.as_deref().and_then(|channel| channel.as_ref().ok());
            let link = details.map(|channel| channel.link()).unwrap_or_default();
            let description = details
                .map(|channel| channel.description())
                .unwrap_or_default();
            let episodes = details
                .map(|channel| channel.episodes())
                .unwrap_or_default();

            let link = format!(
                "<a href=\"{}\">{}</a>",
                escape_str_attribute(&link),
                escape_str_pcdata(&link),
            );

            let title_link = format!(
                "<a href=\"{}\">{} by {}</a>",
                escape_str_attribute(&link),
                escape_str_pcdata(&title),
                escape_str_pcdata(&author),
            );

            gtk! {
                <GtkBox orientation=Orientation::Vertical vexpand=true hexpand=true hexpand=true halign=Align::Fill>
                    <Squeezer homogeneous=false halign=Align::Fill hexpand=true>
                        <GtkBox
                            orientation=Orientation::Vertical
                            border_width=10
                            hexpand=true
                            halign=Align::Fill
                            property_width_request=600
                        >
                            <GtkBox orientation=Orientation::Horizontal>
                                <GtkBox
                                    halign=Align::Start
                                    valign=Align::Start
                                    property_width_request=300
                                    margin_end=10
                                >
                                    <@FixedImage
                                        image={big_image.clone()}
                                        width=300
                                    />
                                </GtkBox>
                                <GtkBox orientation=Orientation::Vertical>
                                    <Label
                                        label=title.to_owned()
                                        hexpand=true
                                        line_wrap=true
                                        max_width_chars=1
                                        lines=2
                                        xalign=0.0
                                        hexpand=true
                                        halign=Align::Fill
                                        valign=Align::Start
                                        ellipsize=EllipsizeMode::End
                                        attributes={Some(desk_title_style.clone())}
                                    />
                                    <Label
                                        label=author.to_string()
                                        visible=!author.is_empty()
                                        hexpand=true
                                        line_wrap=true
                                        max_width_chars=1
                                        lines=2
                                        xalign=0.0
                                        halign=Align::Fill
                                        valign=Align::Start
                                        ellipsize=EllipsizeMode::End
                                    />
                                    <Label
                                        markup=link.clone()
                                        visible=!link.is_empty()
                                        margin_bottom=10
                                        hexpand=true
                                        max_width_chars=1
                                        xalign=0.0
                                        halign=Align::Fill
                                        valign=Align::Start
                                        ellipsize=EllipsizeMode::End
                                    />
                                    <GtkBox margin_bottom=10 />
                                    <Label
                                        label=description.clone()
                                        hexpand=true
                                        line_wrap=true
                                        max_width_chars=1
                                        xalign=0.0
                                        halign=Align::Fill
                                        valign=Align::Start
                                        lines=7
                                        ellipsize=EllipsizeMode::End
                                    />
                                </GtkBox>
                            </GtkBox>
                        </GtkBox>
                        <GtkBox
                            orientation=Orientation::Vertical
                            border_width=10
                            hexpand=true
                            property_width_request=400
                        >
                            <GtkBox halign=Align::Center property_width_request=300 margin_bottom=10>
                                <@FixedImage
                                    image={big_image.clone()}
                                    width=300
                                />
                            </GtkBox>
                            <Label
                                markup=title_link
                                hexpand=true
                                line_wrap=true
                                max_width_chars=1
                                lines=2
                                xalign=0.5
                                halign=Align::Fill
                                valign=Align::Start
                                ellipsize=EllipsizeMode::End
                                attributes={Some(med_title_style.clone())}
                            />
                            <GtkBox margin_bottom=10 />
                            <Label
                                label=description.clone()
                                hexpand=true
                                line_wrap=true
                                max_width_chars=1
                                xalign=0.0
                                halign=Align::Fill
                                hexpand=true
                                valign=Align::Start
                                lines=7
                                ellipsize=EllipsizeMode::End
                            />
                        </GtkBox>
                        <GtkBox
                            orientation=Orientation::Vertical
                            border_width=10
                            property_width_request=300
                        >
                            <GtkBox orientation=Orientation::Horizontal margin_bottom=10>
                                <GtkBox
                                    halign=Align::Start
                                    valign=Align::Start
                                    property_width_request=100
                                    margin_end=10
                                >
                                    <@FixedImage
                                        image={big_image}
                                        width=100
                                    />
                                </GtkBox>
                                <GtkBox orientation=Orientation::Vertical hexpand=true>
                                    <Label
                                        label=title.to_owned()
                                        hexpand=true
                                        line_wrap=true
                                        max_width_chars=1
                                        lines=2
                                        xalign=0.0
                                        halign=Align::Fill
                                        valign=Align::Start
                                        ellipsize=EllipsizeMode::End
                                        attributes={Some(med_title_style.clone())}
                                    />
                                    <Label
                                        label=author.to_string()
                                        visible=!author.is_empty()
                                        hexpand=true
                                        line_wrap=true
                                        max_width_chars=1
                                        lines=2
                                        xalign=0.0
                                        halign=Align::Fill
                                        valign=Align::Start
                                        ellipsize=EllipsizeMode::End
                                    />
                                    <Label
                                        markup=link.clone()
                                        visible=!link.is_empty()
                                        margin_bottom=10
                                        hexpand=true
                                        max_width_chars=1
                                        xalign=0.0
                                        halign=Align::Fill
                                        valign=Align::Start
                                        ellipsize=EllipsizeMode::End
                                    />
                                </GtkBox>
                            </GtkBox>
                            <Label
                                label=description.clone()
                                hexpand=true
                                line_wrap=true
                                max_width_chars=1
                                xalign=0.0
                                halign=Align::Fill
                                valign=Align::Start
                                lines=4
                                ellipsize=EllipsizeMode::End
                            />
                        </GtkBox>
                    </Squeezer>

                    <GtkBox border_width=10 valign=Align::Start hexpand=true orientation=Orientation::Vertical>
                        <ListBox border_width=10 valign=Align::Start hexpand=true selection_mode=SelectionMode::None>
                            {
                                episodes.iter().take(self.episode_limit).enumerate().map(|(i, episode_ref)| {
                                    let episode = episode_ref.get();
                                    let episode = episode.as_deref().and_then(|episode| episode.as_ref().ok());
                                    let title = episode.map(|episode| episode.title()).unwrap_or_default();
                                    let date = episode.map(|episode| episode.date()).unwrap_or_default();
                                    let date = DateTime::parse_from_rfc2822(date).map(|date| date.format("%F").to_string()).unwrap_or_default();
                                    let duration = episode.map(|episode| episode.duration()).unwrap_or_default();
                                    let duration = if duration.contains(":") {
                                        duration.to_owned()
                                    } else {
                                        let seconds = duration.parse::<u32>().unwrap_or(0);
                                        let time = NaiveTime::from_num_seconds_from_midnight(seconds, 0);
                                        if seconds >= 3600 {
                                            time.format("%T").to_string()
                                        } else {
                                            time.format("%M∶%S").to_string()
                                        }
                                    };
                                    let description = episode.map(|episode| episode.description()).unwrap_or_default();

                                    gtk! {
                                        <ListBoxRow activatable=true>
                                            <GtkBox hexpand=false orientation=Orientation::Horizontal>
                                                <GtkBox orientation=Orientation::Vertical hexpand=true>
                                                    <Label
                                                        label=title.to_string()
                                                        line_wrap=true
                                                        max_width_chars=1
                                                        xalign=0.0
                                                        hexpand=true
                                                        halign=Align::Fill
                                                        valign=Align::Start
                                                        attributes={Some(episode_title_style.clone())}
                                                    />
                                                    <Label
                                                        label=format!("{} \u{00B7} {}", date, duration)
                                                        halign=Align::Start
                                                        valign=Align::Start
                                                        attributes={Some(episode_meta_style.clone())}
                                                    />
                                                    <Label
                                                        label=description.replace("\n", " ")
                                                        hexpand=true
                                                        max_width_chars=1
                                                        xalign=0.0
                                                        halign=Align::Fill
                                                        valign=Align::Start
                                                        ellipsize=EllipsizeMode::End
                                                    />
                                                </GtkBox>
                                                <Button
                                                    margin_start=10
                                                    border_width=10
                                                    image="media-playback-start-symbolic"
                                                    halign=Align::End
                                                    valign=Align::Center
                                                    on clicked=|_| {
                                                        Message::HandlePlay(i)
                                                    }
                                                />
                                            </GtkBox>
                                        </ListBoxRow>
                                    }
                                })
                            }
                        </ListBox>
                        {if episodes.len() > self.episode_limit {
                            gtk! {
                                <Button
                                    label="Show more"
                                    visible={episodes.len() > self.episode_limit}
                                    on clicked=|_| Message::HandleShowMore
                                />
                            }
                        } else {
                            gtk! { <Label /> }
                        }}
                    </GtkBox>
                </GtkBox>
            }
        } else {
            gtk! { <GtkBox /> }
        };

        gtk! {
            <ScrolledWindow property_width_request=300 hexpand=true halign=Align::Fill valign=Align::Fill>
                <Viewport hexpand=true halign=Align::Fill>
                    <Column halign=Align::Fill maximum_width=1200 linear_growth_width=1200 hexpand=true>
                        {contents}
                    </Column>
                </Viewport>
            </ScrolledWindow>
        }
    }
}
