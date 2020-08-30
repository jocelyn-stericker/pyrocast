use async_std::stream::StreamExt;
use loader::{Loader, Query};
use state::{CurrentState, StateAction};

// For now, this is a way for me to sanity check things, not actually a useful CLI.
fn main() {
    async_std::task::block_on(async {
        let (current, mut waiter) = CurrentState::new();
        let loader = Loader::new(current.clone(), 10);

        loader.queue(Query::ItunesChart {});
        current.update(vec![StateAction::SetSearchQuery(String::from(
            "The Journal",
        ))]);
        loader.queue(Query::ItunesSearch {
            query: String::from("The Journal"),
        });

        while waiter.next().await.is_some() {
            print!(".");
            if !current.get().loading() {
                break;
            }
        }

        let state = current.get();

        let channel_result = state
            .search_results()
            .unwrap()
            .first()
            .unwrap()
            .core()
            .unwrap();
        let channel = Result::as_ref(&channel_result).unwrap();

        let rss = channel.rss().unwrap();
        loader.queue(Query::Rss {
            pk: channel.pk().to_owned(),
            url: rss.to_owned(),
        });

        while waiter.next().await.is_some() {
            print!(".");
            if !current.get().loading() {
                break;
            }
        }
        println!("{:#?}", current.get());
    });
}
