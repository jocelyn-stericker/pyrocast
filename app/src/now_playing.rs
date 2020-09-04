use crate::fixed_image::FixedImage;
use crate::vgtk_ext::*;
use chrono::NaiveTime;
use libhandy::{Column, ColumnExt};
use pango::EllipsizeMode;
use pango::{AttrList, Attribute, Weight};
use state::{Episode, Playback, PlayerState, StateError};
use std::sync::Arc;
use vgtk::lib::gtk::{
    prelude::*, Align, Box, Button, ComboBoxText, Label, Orientation, Scale, ScrolledWindow,
    Spinner,
};
use vgtk::{gtk, Callback, Component, UpdateAction, VNode};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Props {
    pub player_state: Arc<Option<PlayerState>>,
    pub episode_info: Option<Arc<Result<Episode, StateError>>>,
    pub on_skip_back: Callback<()>,
    pub on_skip_forward: Callback<()>,
    pub on_pause: Callback<()>,
    pub on_unpause: Callback<()>,
    pub on_seek: Callback<u64>,
    pub on_rate_change: Callback<f64>,
}

#[derive(Debug, Default, Clone)]
pub struct NowPlaying {
    props: Props,
}

#[derive(Clone, Debug)]
pub enum Message {
    HandleSkipBack,
    HandleSkipForward,
    HandlePause,
    HandleUnpause,
    HandleMaybeSeek(f64),
    HandleSetRate(f64),
    None,
}

impl Component for NowPlaying {
    type Message = Message;
    type Properties = Props;

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::HandleSkipBack => {
                self.props.on_skip_back.send(());
                UpdateAction::None
            }
            Message::HandleSkipForward => {
                self.props.on_skip_forward.send(());
                UpdateAction::None
            }
            Message::HandlePause => {
                self.props.on_pause.send(());
                UpdateAction::None
            }
            Message::HandleUnpause => {
                self.props.on_unpause.send(());
                UpdateAction::None
            }
            Message::HandleMaybeSeek(time) => {
                let curr_time = self
                    .props
                    .player_state
                    .as_ref()
                    .as_ref()
                    .map(|state| state.time as f64)
                    .unwrap_or_default();

                if (time - curr_time as f64).abs() > 1000.0 {
                    self.props.on_seek.send(time as u64);
                }

                UpdateAction::None
            }
            Message::HandleSetRate(rate) => {
                self.props.on_rate_change.send(rate);

                UpdateAction::None
            }
            Message::None => UpdateAction::None,
        }
    }

    fn change(&mut self, props: Self::Properties) -> UpdateAction<Self> {
        if self.props != props {
            self.props = props;
            UpdateAction::Render
        } else {
            UpdateAction::None
        }
    }

    fn view(&self) -> VNode<NowPlaying> {
        let episode = &self.props.episode_info;
        let episode = episode.as_deref();
        let episode = episode.and_then(|ep| ep.as_ref().ok());

        let player_state = self.props.player_state.as_ref().as_ref();
        let time = player_state
            .map(|state| state.time as f64)
            .unwrap_or_default();
        let duration = player_state
            .map(|state| state.duration as f64)
            .unwrap_or_default();
        let rate = player_state
            .map(|state| state.rate as f64)
            .unwrap_or_default();
        let playback = player_state
            .map(|state| state.playback)
            .unwrap_or(Playback::Stopped);

        let med_title_style = AttrList::new();
        med_title_style.insert(Attribute::new_scale(1.2).unwrap());
        med_title_style.insert(Attribute::new_weight(Weight::Bold).unwrap());

        let speeds: Vec<(String, String)> = vec![
            ("0.5".to_owned(), "0.5x".to_owned()),
            ("1".to_owned(), "Standard speed".to_owned()),
            ("1.5".to_owned(), "1.5x".to_owned()),
            ("2".to_owned(), "2.0x".to_owned()),
            ("3".to_owned(), "3.0x".to_owned()),
        ];

        gtk! {
            <Column hexpand=true maximum_width=768 halign=Align::Fill>
                <Box orientation=Orientation::Vertical hexpand=true vexpand=true halign=Align::Fill valign=Align::Center>
                    <ScrolledWindow vexpand=true hexpand=true propagate_natural_height=true propagate_natural_width=true>
                        <Box
                            orientation=Orientation::Vertical
                            hexpand=true
                            vexpand=true
                            halign=Align::Fill
                            valign=Align::Center
                            border_width=10
                        >
                            <@FixedImage
                                image=episode.and_then(|ep| ep.image())
                                width=300
                            />
                            <Label
                                margin_top=10
                                label=episode.map(|ep| ep.title().to_owned()).unwrap_or_default()
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
                                label=episode.map(|ep| ep.description().to_owned()).unwrap_or_default()
                                hexpand=true
                                line_wrap=true
                                max_width_chars=1
                                xalign=0.0
                                halign=Align::Fill
                                valign=Align::Start
                                lines=7
                                ellipsize=EllipsizeMode::End
                            />
                            <ComboBoxText
                                hexpand=false
                                halign=Align::Start
                                margin_top=10
                                options=speeds
                                active_id=rate.to_string()
                                on property_active_id_notify=|widget| {
                                    Message::HandleSetRate(
                                        widget
                                            .get_active_id()
                                            .and_then(|rate|
                                                rate.to_string().parse::<f64>().ok())
                                            .unwrap_or(1.0)
                                    )
                                }
                            />
                        </Box>
                    </ScrolledWindow>
                    <Box orientation=Orientation::Vertical hexpand=true vexpand=true halign=Align::Fill valign=Align::Center>
                        <Scale
                            margin_top=10
                            hexpand=true
                            valign=Align::Center
                            value={time}
                            range_pair=(0.0f64, duration as f64)
                            sensitive=playback.active()
                            property_width_request=300
                            on show=|widget| {
                                widget.connect_format_value(|_, val| {
                                    // Trick from
                                    // https://gitlab.gnome.org/World/podcasts/-/blob/817203158b9d3736880e08969f406dc7d1d4ebb4/podcasts-gtk/src/widgets/player.rs#L199
                                    let seconds = (val.max(0.0) as u32) / 1000;
                                    let time = NaiveTime::from_num_seconds_from_midnight(seconds, 0);
                                    if seconds >= 3600 {
                                        time.format("%T").to_string()
                                    } else {
                                        time.format("%M∶%S").to_string()
                                    }
                                });

                                Message::None
                            }
                            on value_changed=|range| {
                                Message::HandleMaybeSeek(range.get_value())
                            }
                        />
                        <Box halign=Align::Center valign=Align::Center margin_top=10>
                            <Button
                                image="media-seek-backward-symbolic"
                                on clicked=|_| Message::HandleSkipBack
                                border_width=10
                                sensitive=playback.active()
                            />
                            {
                                match playback {
                                    Playback::Stopped => {
                                        gtk! {
                                            <Button
                                                image="media-playback-start-symbolic"
                                                on clicked=|_| Message::HandleUnpause
                                                border_width=10
                                                sensitive=false
                                                property_width_request=80
                                            />
                                        }
                                    }
                                    Playback::Buffering => {
                                        gtk! { <Spinner /> }
                                    }
                                    Playback::Paused => {
                                        gtk! {
                                            <Button
                                                image="media-playback-start-symbolic"
                                                on clicked=|_| Message::HandleUnpause
                                                border_width=10
                                                sensitive=true
                                                property_width_request=80
                                            />
                                        }
                                    }
                                    Playback::Playing => {
                                        gtk! {
                                            <Button
                                                image="media-playback-pause-symbolic"
                                                on clicked=|_| Message::HandlePause
                                                border_width=10
                                                sensitive=true
                                                property_width_request=80
                                            />
                                        }
                                    }
                                }
                            }
                            <Button
                                image="media-seek-forward-symbolic"
                                on clicked=|_| Message::HandleSkipForward
                                border_width=10
                                sensitive=playback.active()
                            />
                        </Box>
                    </Box>
                </Box>
            </Column>
        }
    }
}
