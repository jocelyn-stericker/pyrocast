mod itunes_channel;
mod itunes_chart;
mod itunes_lookup;
mod itunes_search;
mod query;
mod rss;

use async_std::task;
use itunes_channel::ItunesChannel;
use itunes_chart::ItunesChart;
use itunes_lookup::ItunesLookup;
use itunes_search::ItunesSearch;
pub use query::Query;
use rss::Rss;
use state::{CurrentState, Image, StateAction};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use surf::http_types::StatusCode;
use surf::mime::STAR_STAR;

async fn handle_query(current: &CurrentState, query: &Query, request_idx: usize) -> Vec<Query> {
    let state = current.get();
    match query {
        Query::ItunesChart => {
            let country = state.country();
            let explicit = state.allow_explicit();
            match ItunesChart::fetch(country, explicit, 100).await {
                Ok(chart) => {
                    current.update(chart.to_actions(current));

                    let mut next_requests = vec![];
                    for item in chart.feed.results {
                        next_requests.push(Query::Image {
                            image: Arc::new(Image::new(&item.image_200)),
                            associated_query: Some(request_idx),
                        });
                    }

                    next_requests
                }
                Err(err) => {
                    current.update(vec![StateAction::SetSearchFeed {
                        query: String::new(),
                        results: Err(err),
                    }]);

                    Vec::new()
                }
            }
        }
        Query::ItunesSearch { query } => {
            let country = state.country();
            let explicit = state.allow_explicit();
            match ItunesSearch::fetch(&country, explicit, 50, query).await {
                Ok(results) => {
                    current.update(results.to_actions(current));

                    let mut next_requests = vec![];
                    for item in results.results {
                        next_requests.push(Query::Image {
                            image: Arc::new(Image::new(&item.image_600)),
                            associated_query: Some(request_idx),
                        });
                    }

                    next_requests
                }
                Err(err) => {
                    current.update(vec![StateAction::SetSearchFeed {
                        query: String::from(query),
                        results: Err(err),
                    }]);

                    Vec::new()
                }
            }
        }
        Query::ItunesLookup { pk } => match pk.parse::<isize>() {
            Ok(id) => match ItunesLookup::fetch(id).await {
                Ok(lookup) => {
                    current.update(lookup.to_actions(current));
                    lookup
                        .cores(current)
                        .iter()
                        .filter_map(|core| {
                            Some(Query::Rss {
                                pk: core.pk().to_owned(),
                                url: core.rss()?.to_owned(),
                            })
                        })
                        .collect()
                }
                Err(err) => {
                    current.update(vec![
                        StateAction::SetChannelCore(pk.to_owned(), Err(err.clone())),
                        StateAction::SetChannelDetail(pk.to_owned(), Err(err)),
                    ]);
                    vec![]
                }
            },
            Err(_err) => {
                let err = std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{} is not an iTunes id (integer)", &pk),
                );
                // No clone
                let err2 = std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{} is not an iTunes id (integer)", &pk),
                );
                current.update(vec![
                    StateAction::SetChannelCore(pk.to_owned(), Err(err.into())),
                    StateAction::SetChannelDetail(pk.to_owned(), Err(err2.into())),
                ]);
                vec![]
            }
        },
        Query::Rss { url, pk } => match Rss::fetch(url, pk).await {
            Ok(rss) => {
                current.update(rss.channel.to_actions(current));

                vec![Query::Image {
                    image: Arc::new(Image::new(&rss.channel.image_rss())),
                    associated_query: None,
                }]
            }
            Err(err) => {
                current.update(vec![
                    StateAction::SetChannelCore(pk.to_owned(), Err(err.clone())),
                    StateAction::SetChannelDetail(pk.to_owned(), Err(err)),
                ]);

                vec![]
            }
        },
        Query::Image { image, .. } => {
            let mut request = surf::get(&image.pk);
            if let Some(etag) = &image.etag {
                request = request.set_header("if-none-match", etag.clone());
            }
            if let Some(last_modified) = &image.last_modified {
                request = request.set_header("if-modified-since", last_modified.clone());
            }

            match request.await {
                Ok(mut response) => {
                    if response.status() != StatusCode::NotModified {
                        match response.body_bytes().await {
                            Ok(body) => {
                                let mime = response.mime().unwrap_or(STAR_STAR);
                                current.update(vec![StateAction::SetImage(
                                    image.pk.clone(),
                                    Ok(Image {
                                        pk: image.pk.clone(),
                                        mimetype: Some(mime.to_string()),
                                        etag: Some(
                                            response
                                                .header("etag")
                                                .map(|header| header.last().to_string())
                                                .unwrap_or_default(),
                                        ),
                                        last_modified: Some(
                                            response
                                                .header("last-modified")
                                                .map(|header| header.last().to_string())
                                                .unwrap_or_default(),
                                        ),
                                        data: Some(body),
                                    }),
                                )]);
                            }
                            Err(err) => {
                                current.update(vec![StateAction::SetImage(
                                    image.pk.clone(),
                                    Err(err.into()),
                                )]);
                            }
                        }
                    }
                }
                Err(err) => {
                    current.update(vec![StateAction::SetImage(
                        image.pk.clone(),
                        Err(err.into()),
                    )]);
                }
            }

            Vec::default()
        }
    }
}

#[derive(Debug)]
struct LoaderPriv {
    queries: BinaryHeap<(Query, Reverse<usize>)>,
    in_flight: usize,
    max_in_flight: usize,
    /// Used to break Query priority ties (oldest first).
    total_queries: usize,
    max_search_index: usize,
}

#[derive(Debug, Clone)]
pub struct Loader {
    current: Arc<CurrentState>,
    data: Arc<Mutex<LoaderPriv>>,
}

impl Loader {
    pub fn new(current: Arc<CurrentState>, max_in_flight: usize) -> Loader {
        Loader {
            current,
            data: Arc::new(Mutex::new(LoaderPriv {
                total_queries: 0,
                queries: BinaryHeap::new(),
                in_flight: 0,
                max_search_index: 0,
                max_in_flight,
            })),
        }
    }

    pub fn queue(&self, query: Query) {
        self._queue(query, &mut self.data.lock().unwrap());
    }

    fn _queue(&self, query: Query, data: &mut LoaderPriv) {
        if query.is_search() {
            data.max_search_index = data.total_queries;
        }

        data.queries.push((query, Reverse(data.total_queries)));
        data.total_queries += 1;
        if data.in_flight < data.max_in_flight {
            data.in_flight += 1;

            if data.in_flight == 1 {
                self.current.update(vec![StateAction::SetLoading(true)]);
            }

            eprintln!(
                "Spawn worker, now {}/{}",
                data.in_flight, data.max_in_flight
            );
            self.spawn_worker();
        }
    }

    fn pop(&self) -> Option<(Query, usize)> {
        let mut data = self.data.lock().unwrap();

        while let Some((query, request_idx)) = data.queries.pop() {
            // Skip queries that are now out of date.
            if query.is_search() && request_idx.0 != data.max_search_index {
                continue;
            }

            if matches!(&query, Query::Image {associated_query: Some(image_request_idx), .. } if *image_request_idx < data.max_search_index)
            {
                continue;
            }

            return Some((query, request_idx.0));
        }

        None
    }

    fn spawn_worker(&self) {
        let scheduler = self.clone();
        task::spawn(async move {
            while let Some((query, request_idx)) = scheduler.pop() {
                let now = std::time::Instant::now();
                let next: Vec<Query> = handle_query(&scheduler.current, &query, request_idx).await;
                eprintln!("Task: {:?}", now.elapsed());

                let mut data = scheduler.data.lock().unwrap();
                for q in next {
                    scheduler._queue(q, &mut data);
                }
            }

            let mut data = scheduler.data.lock().unwrap();
            data.in_flight -= 1;
            eprintln!("Free worker, now {}/{}", data.in_flight, data.max_in_flight);

            if data.in_flight == 0 {
                scheduler
                    .current
                    .update(vec![StateAction::SetLoading(false)]);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
