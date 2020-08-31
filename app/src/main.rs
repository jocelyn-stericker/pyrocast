#![recursion_limit = "2048"]
#![allow(clippy::toplevel_ref_arg)]

mod app;
mod fixed_image;
mod home_tab;
mod preferred_size;
mod search_detail;
mod search_results;
mod search_tab;
mod settings_tab;
mod vgtk_ext;

use app::{App, Message};
use async_std::stream::StreamExt;
use async_std::task;
use loader::{Loader, Query};
use state::CurrentState;
use std::env::args;
use std::sync::Arc;
use vgtk::lib::gio::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    pretty_env_logger::init();
    let (app, scope) = vgtk::start::<App>();
    let (current, mut waiter) = CurrentState::new();
    scope.send_message(Message::InitDispatch(Arc::clone(&current)));

    task::spawn(async move {
        let loader = Loader::new(current.clone(), 10);
        scope.send_message(Message::InitLoaderWIP(loader.clone()));

        loader.queue(Query::ItunesChart {});

        while waiter.next().await.is_some() {
            scope.send_message(Message::StateChanged(current.get()));
        }
        drop(scope);
    });

    let ret = app.run(&args().collect::<Vec<_>>());
    eprintln!("App quitting.");
    std::process::exit(ret);
}
