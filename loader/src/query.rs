use state::Image;
use std::cmp::{Ord, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub enum Query {
    ItunesChart,
    ItunesSearch {
        query: String,
    },
    ItunesLookup {
        pk: String,
    },
    Rss {
        pk: String,
        url: String,
    },
    Image {
        image: Arc<Image>,
        associated_query: Option<usize>,
    },
}

impl Query {
    pub fn priority(&self) -> usize {
        match self {
            Query::Rss { .. } => 3000,
            Query::ItunesLookup { .. } => 2000,
            Query::ItunesSearch { .. } => 1001,
            Query::ItunesChart { .. } => 1000,
            Query::Image { image, .. } if !image.loaded() => 500,
            Query::Image { .. } => 1,
        }
    }

    pub fn is_search(&self) -> bool {
        matches!(self, Query::ItunesSearch { .. } | Query::ItunesChart)
    }
}

impl PartialEq for Query {
    fn eq(&self, other: &Query) -> bool {
        self.priority() == other.priority()
    }
}

impl Eq for Query {}

impl Ord for Query {
    fn cmp(&self, other: &Query) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Query {
    fn partial_cmp(&self, other: &Query) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
