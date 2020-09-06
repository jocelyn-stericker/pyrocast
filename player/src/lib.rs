mod desktop;

use crate::desktop::{init_desktop_connection, DesktopAction};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::ClockTime;
use gstreamer_player as gplayer;
use gstreamer_player::PlayerState as GPlayerState;
use state::{CurrentState, Playback, PlayerState, StateAction};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PlayerAction {
    PlayRemote {
        episode_pk: String,
        channel_pk: String,
        uri: String,
    },
    Pause,
    Unpause,
    SeekForward,
    SeekBack,
    SetTime(u64),
    SetRate(f64),
}

fn audio_thread(
    send: Sender<PlayerAction>,
    recv: Receiver<PlayerAction>,
    current: Arc<CurrentState>,
) {
    gst::init().unwrap();

    // TODO: can we reuse vgtk's loop?
    let audio_loop = glib::MainLoop::new(None, false);

    let dispatcher = gplayer::PlayerGMainContextSignalDispatcher::new(None);
    let player = gplayer::Player::new(
        None,
        Some(&dispatcher.upcast::<gplayer::PlayerSignalDispatcher>()),
    );

    // Connect to the player's "end-of-stream" signal, which will tell us when the
    // currently played media stream reached its end.
    player.connect_end_of_stream(move |player| {
        player.stop();
    });

    // Connect to the player's "error" signal, which will inform us about eventual
    // errors (such as failing to retrieve a http stream).
    player.connect_error(move |player, err| {
        eprintln!("{}", err);

        player.stop();
    });

    std::thread::spawn(move || {
        audio_loop.run();
    });

    let mut episode_pk = String::default();
    let mut channel_pk = String::default();
    let rate = Arc::new(Mutex::new(1.0));
    let gplayer_state = Arc::new(Mutex::new(GPlayerState::Stopped));

    let rate_clone = Arc::clone(&rate);
    let gplayer_state_clone = Arc::clone(&gplayer_state);

    let desktop_to_player = Arc::new(Mutex::new(Some(send)));
    let player_to_desktop = Arc::new(Mutex::new(None));

    let current_clone = current.clone();
    player.connect_state_changed(move |player, state| {
        let rate = rate_clone.lock().unwrap();
        if state == GPlayerState::Playing && (*rate - player.get_rate()).abs() > std::f64::EPSILON {
            player.set_rate(*rate);
        }

        *gplayer_state_clone.lock().unwrap() = state;

        let mut desktop_to_player = desktop_to_player.lock().unwrap();
        if let Some(send) = desktop_to_player.take() {
            let to_desktop = init_desktop_connection(send);
            *player_to_desktop.lock().unwrap() = Some(to_desktop);
        }

        if let Some(send) = &*player_to_desktop.lock().unwrap() {
            match state {
                GPlayerState::Playing => send
                    .send(DesktopAction::Play {
                        artist: current_clone
                            .get()
                            .playing_episode()
                            .as_deref()
                            .and_then(|ep| ep.as_ref().ok())
                            .and_then(|ep| ep.channel().core())
                            .as_deref()
                            .and_then(|chan| chan.as_ref().ok())
                            .map(|chan| chan.title().to_owned())
                            .unwrap_or_default(),
                        title: current_clone
                            .get()
                            .playing_episode()
                            .as_deref()
                            .and_then(|ep| ep.as_ref().ok())
                            .map(|ep| ep.title().to_owned())
                            .unwrap_or_default(),
                    })
                    .unwrap(),
                GPlayerState::Paused => send.send(DesktopAction::Pause).unwrap(),
                GPlayerState::Stopped => send.send(DesktopAction::Stop).unwrap(),
                _ => {}
            }
        }
    });

    // Time is 0 while buffering.
    let mut last_known_time = 0;

    loop {
        match recv.recv_timeout(Duration::from_millis(100)) {
            Ok(PlayerAction::PlayRemote {
                episode_pk: new_episode_pk,
                channel_pk: new_channel_pk,
                uri,
            }) => {
                player.set_uri(&uri);
                // Can only set rate in playing state.
                player.set_rate(1.0);
                player.play();
                episode_pk = new_episode_pk;
                channel_pk = new_channel_pk;
            }
            Ok(PlayerAction::Pause) => {
                player.pause();
            }
            Ok(PlayerAction::Unpause) => {
                player.play();
            }
            Ok(PlayerAction::SeekForward) => {
                last_known_time += 30000;
                player.seek(ClockTime::from_mseconds(last_known_time));
            }
            Ok(PlayerAction::SeekBack) => {
                last_known_time = ((last_known_time as i64) - 15000).max(0) as u64;
                player.seek(ClockTime::from_mseconds(last_known_time));
            }
            Ok(PlayerAction::SetTime(t)) => {
                last_known_time = t;
                player.seek(ClockTime::from_mseconds(t));
            }
            Ok(PlayerAction::SetRate(next_rate)) => {
                *rate.lock().unwrap() = next_rate;
                if *gplayer_state.lock().unwrap() == GPlayerState::Playing {
                    player.set_rate(next_rate);
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // we'll update the status.
            }
            Err(RecvTimeoutError::Disconnected) => {
                return;
            }
        }

        let playback = match *gplayer_state.lock().unwrap() {
            GPlayerState::Playing => Playback::Playing,
            GPlayerState::Stopped => Playback::Stopped,
            GPlayerState::Paused => Playback::Paused,
            GPlayerState::Buffering => Playback::Buffering,
            _ => Playback::Buffering,
        };

        if playback != Playback::Buffering {
            last_known_time = player.get_position().mseconds().unwrap_or(0);
        }

        current.update(vec![StateAction::SetPlayerState(Some(PlayerState {
            episode_pk: episode_pk.clone(),
            channel_pk: channel_pk.clone(),
            playback,
            time: last_known_time,
            duration: player.get_duration().mseconds().unwrap_or(0),
            rate: player.get_rate(),
        }))]);
    }
}

pub fn new_player(current: Arc<CurrentState>) -> Sender<PlayerAction> {
    let (send_cmd, recv_cmd) = channel();
    let send_cmd_clone = send_cmd.clone();
    std::thread::spawn(move || {
        audio_thread(send_cmd_clone, recv_cmd, current);
    });

    send_cmd
}
