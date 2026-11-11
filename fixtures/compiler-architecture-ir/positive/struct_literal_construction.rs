pub mod adapters {
    pub struct Http {
        pub base: String,
    }

    pub struct SystemClock;

    pub enum Transport {
        Loopback,
    }

    pub struct SqlRepo {
        url: String,
    }

    pub struct ConnectError;

    impl SqlRepo {
        pub fn connect(url: &str) -> Result<Self, ConnectError> {
            Ok(Self { url: url.to_owned() })
        }

        pub async fn connect_async(url: &str) -> Result<Self, ConnectError> {
            Self::connect(url)
        }

        pub fn open(url: &str) -> Option<Box<Self>> {
            Self::connect(url).ok().map(Box::new)
        }

        pub fn url(&self) -> &str {
            &self.url
        }
    }

    pub fn shared(url: &str) -> Option<std::sync::Arc<SqlRepo>> {
        SqlRepo::connect(url).ok().map(std::sync::Arc::new)
    }
}

pub fn compose(base: String) -> (adapters::Http, adapters::SystemClock, adapters::Transport) {
    (adapters::Http { base }, adapters::SystemClock, adapters::Transport::Loopback)
}

pub fn compose_fallible(url: &str) -> Result<usize, adapters::ConnectError> {
    let repo = adapters::SqlRepo::connect(url)?;
    Ok(repo.url().len())
}

pub fn compose_boxed(url: &str) -> usize {
    adapters::SqlRepo::open(url).map_or(0, |repo| repo.url().len())
}

pub fn compose_shared(url: &str) -> usize {
    adapters::shared(url).map_or(0, |repo| repo.url().len())
}

pub async fn compose_async(url: &str) -> Result<usize, adapters::ConnectError> {
    let repo = adapters::SqlRepo::connect_async(url).await?;
    Ok(repo.url().len())
}
