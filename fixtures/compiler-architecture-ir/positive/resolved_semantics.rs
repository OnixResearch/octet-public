pub trait Repository {
    fn load(&self, key: &str) -> Option<String>;
}

pub type RepositoryResult<T> = Option<T>;

pub struct MemoryRepository;

impl Repository for MemoryRepository {
    fn load(&self, key: &str) -> RepositoryResult<String> {
        Some(key.to_owned())
    }
}

macro_rules! repository_key {
    ($value:expr) => {
        $value
    };
}

pub fn build_repository() -> impl Repository {
    MemoryRepository
}

pub fn run<T>(repository: &impl Repository, key: T) -> RepositoryResult<String>
where
    T: AsRef<str>,
{
    repository.load(repository_key!(key.as_ref()))
}
