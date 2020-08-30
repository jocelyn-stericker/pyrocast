use async_std::stream::StreamExt;
use loader::{Loader, Query};
use player::PlayerAction;
use state::{CurrentState, StateAction};

// For now, this is a way for me to sanity check things, not actually a useful CLI.
fn main() {
    async_std::task::block_on(async {
        let (current, mut waiter) = CurrentState::new();
        let loader = Loader::new(current.clone(), 10);

        // loader.queue(Query::ItunesChart {});
        // current.update(vec![StateAction::SetSearchQuery(String::from(
        //     "The Journal",
        // ))]);
        // loader.queue(Query::ItunesSearch {
        //     query: String::from("The Journal"),
        // });

        // while waiter.next().await.is_some() {
        //     print!(".");
        //     if !current.get().loading() {
        //         break;
        //     }
        // }

        // let state = current.get();

        // let channel_result = state
        //     .search_results()
        //     .unwrap()
        //     .first()
        //     .unwrap()
        //     .core()
        //     .unwrap();
        // let channel = Result::as_ref(&channel_result).unwrap();

        // let rss = channel.rss().unwrap();
        // loader.queue(Query::Rss {
        //     pk: channel.pk().to_owned(),
        //     url: rss.to_owned(),
        // });

        // while waiter.next().await.is_some() {
        //     print!(".");
        //     if !current.get().loading() {
        //         break;
        //     }
        // }
        // println!("{:#?}", current.get());

        // let state = current.get();

        // let episode = state
        //     .search_results()
        //     .unwrap()
        //     .first()
        //     .unwrap()
        //     .details()
        //     .unwrap()
        //     .as_ref()
        //     .as_ref()
        //     .unwrap()
        //     .episodes()
        //     .last()
        //     .unwrap()
        //     .get()
        //     .unwrap();

        // let episode = episode.as_ref().as_ref().unwrap();

        let pa = player::new_player(current.clone());
        pa.send(PlayerAction::SetRate(2.0)).unwrap();
        pa.send(PlayerAction::PlayRemote {
            episode_pk: "1234".to_owned(), // episode.pk().to_owned(),
            channel_pk: "1234".to_owned(),
            uri: "file:///tmp/a.mp3".to_owned(), // episode.audio().to_owned(),
        })
        .unwrap();

        while waiter.next().await.is_some() {
            println!("{:#?}", current.get().player_state());
        }
    });
}
