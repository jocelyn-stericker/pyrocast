use crate::PlayerAction;
use glib::Cast;
use gtk::{ApplicationInhibitFlags, GtkApplicationExt};
use mpris_player::{Metadata, MprisPlayer, OrgMprisMediaPlayer2Player, PlaybackStatus};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub(crate) enum DesktopAction {
    Play { artist: String, title: String },
    Pause,
    Stop,
}

pub(crate) fn init_desktop_connection(send: Sender<PlayerAction>) -> glib::Sender<DesktopAction> {
    let mpris = MprisPlayer::new(
        "ca.nettek.pyrocast".to_owned(),
        "Pyrocast".to_owned(),
        "ca.nettek.pyrocast.desktop".to_owned(),
    );
    mpris.set_can_play(true);
    mpris.set_can_pause(true);
    mpris.set_can_go_next(true);
    mpris.set_can_go_previous(true);

    let mpris_clone = mpris.clone();
    let send_clone = send.clone();
    mpris.connect_play_pause(move || {
        if let Ok(status) = mpris_clone.get_playback_status() {
            match status.as_ref() {
                "Paused" | "Stopped" => send_clone.send(PlayerAction::Unpause).unwrap(),
                _ => send_clone.send(PlayerAction::Pause).unwrap(),
            };
        }
    });

    let send_clone = send.clone();
    mpris.connect_play(move || {
        send_clone.send(PlayerAction::Unpause).unwrap();
    });

    let send_clone = send.clone();
    mpris.connect_pause(move || {
        send_clone.send(PlayerAction::Pause).unwrap();
    });

    let send_clone = send.clone();
    mpris.connect_next(move || {
        send_clone.send(PlayerAction::SeekForward).unwrap();
    });

    let send_clone = send;
    mpris.connect_previous(move || {
        send_clone.send(PlayerAction::SeekBack).unwrap();
    });

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let inhibit_cookie = Arc::new(Mutex::new(None));

    rx.attach(None, move |command| {
        // TODO: seek
        // TODO: image
        match command {
            DesktopAction::Play { artist, title } => {
                if let Some(app) = gio::Application::get_default()
                    .and_then(|app| app.downcast::<gtk::Application>().ok())
                {
                    eprintln!("Inhibiting suspend");
                    *inhibit_cookie.lock().unwrap() = Some(app.inhibit(
                        app.get_active_window().as_ref(),
                        ApplicationInhibitFlags::SUSPEND,
                        Some("podcast playing"),
                    ));
                }
                let mut metadata = Metadata::new();
                metadata.artist = Some(vec![artist]);
                metadata.title = Some(title);
                mpris.set_metadata(metadata);

                mpris.set_playback_status(PlaybackStatus::Playing);
            }
            DesktopAction::Pause => {
                mpris.set_playback_status(PlaybackStatus::Paused);

                if let Some(app) = gio::Application::get_default()
                    .and_then(|app| app.downcast::<gtk::Application>().ok())
                {
                    if let Some(cookie) = inhibit_cookie.lock().unwrap().take() {
                        eprintln!("Uninhibiting suspend");
                        app.uninhibit(cookie);
                    }
                }
            }
            DesktopAction::Stop => {
                mpris.set_playback_status(PlaybackStatus::Stopped);
                mpris.set_metadata(Metadata::new());

                if let Some(app) = gio::Application::get_default()
                    .and_then(|app| app.downcast::<gtk::Application>().ok())
                {
                    if let Some(cookie) = inhibit_cookie.lock().unwrap().take() {
                        eprintln!("Uninhibiting suspend");
                        app.uninhibit(cookie);
                    }
                }
            }
        }

        // Tell glib not to remove our callback
        glib::Continue(true)
    });

    tx
}
