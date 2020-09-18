#![recursion_limit = "2048"]
#![allow(clippy::toplevel_ref_arg)]

mod app;
mod fixed_image;
mod home_tab;
mod now_playing;
mod preferred_size;
mod search_detail;
mod search_results;
mod search_tab;
mod settings_tab;
mod vgtk_ext;

use app::{App, Message};
use async_std::stream::StreamExt;
use async_std::task;
use loader::Loader;
use state::CurrentState;
use std::env::args;
use std::sync::Arc;
use vgtk::lib::gio::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    pretty_env_logger::init();

    let (app, scope) = vgtk::start::<App>();

    vgtk::lib::glib::set_prgname(Some("ca.nettek.pyrocast"));

    let (current, mut waiter) = CurrentState::new();
    scope.send_message(Message::InitDispatch(Arc::clone(&current)));

    task::spawn(async move {
        let loader = Loader::new(current.clone(), 10);
        let player = player::new_player(current.clone());
        let database = database::new_database(current.clone(), loader.clone());

        scope.send_message(Message::Init(loader, player, database));

        while waiter.next().await.is_some() {
            let sent = scope.try_send(Message::StateChanged(current.get()));
            if sent.is_err() {
                break;
            }
        }
        drop(scope);
    });

    let ret = app.run(&args().collect::<Vec<_>>());
    eprintln!("App quitting.");
    std::process::exit(ret);
}
