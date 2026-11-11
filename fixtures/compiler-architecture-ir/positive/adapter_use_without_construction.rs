use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

pub mod adapters {
    #[derive(Clone, Default)]
    pub struct SqlRepo {
        url: String,
    }

    impl SqlRepo {
        pub fn connect(url: &str) -> Self {
            Self { url: url.to_owned() }
        }

        pub fn url(&self) -> &str {
            &self.url
        }
    }

    impl From<&str> for SqlRepo {
        fn from(url: &str) -> Self {
            Self::connect(url)
        }
    }

    impl std::str::FromStr for SqlRepo {
        type Err = std::convert::Infallible;

        fn from_str(url: &str) -> Result<Self, Self::Err> {
            Ok(Self::connect(url))
        }
    }
}

use adapters::SqlRepo;

pub fn use_shared(repo: &Arc<Mutex<SqlRepo>>) -> usize {
    let shared = Arc::clone(repo);
    let guard = shared.lock().unwrap();
    guard.url().len()
}

pub fn use_cloned(repo: &SqlRepo) -> usize {
    repo.clone().url().len()
}

pub fn use_cell(repo: &RefCell<SqlRepo>) -> usize {
    repo.borrow().url().len()
}

pub fn use_pinned(repo: Pin<&mut SqlRepo>) -> usize {
    repo.into_ref().get_ref().url().len()
}

pub fn use_slice(repos: &[SqlRepo]) -> usize {
    repos.iter().map(|repo| repo.url().len()).sum()
}

pub fn use_owned(mut repos: Vec<SqlRepo>, spare: Option<SqlRepo>) -> usize {
    let mut kept: Vec<SqlRepo> = Vec::new();
    let none: Option<SqlRepo> = None;
    let last = repos.pop();
    let mut total = 0;
    for repo in &repos {
        total += repo.url().len();
    }
    for repo in repos {
        kept.push(repo);
    }
    let by_name: HashMap<String, SqlRepo> = HashMap::new();
    total + last.or(spare).or(none).map_or(0, |repo| repo.url().len()) + kept.len() + by_name.len()
}

pub fn construct_direct(url: &str) -> usize {
    let boxed = Box::new(SqlRepo::connect(url));
    boxed.url().len() + SqlRepo::default().url().len()
}

pub fn make<T: Default>() -> T {
    T::default()
}

pub fn construct_generic(url: &str) -> usize {
    let converted: SqlRepo = url.into();
    let parsed: SqlRepo = url.parse().unwrap_or_default();
    converted.url().len() + parsed.url().len() + make::<SqlRepo>().url().len()
}
