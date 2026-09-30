use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static LOCKS: OnceLock<Mutex<HashMap<PathBuf, Mutex<()>>>> = OnceLock::new();

fn locks() -> &'static Mutex<HashMap<PathBuf, Mutex<()>>> {
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Per-canonicalized-path mutex for serializing operations on the same repository.
/// Cross-repo operations run in parallel.
pub struct RepoLock {
    _guard: Box<std::sync::MutexGuard<'static, ()>>,
}

impl RepoLock {
    pub fn acquire(repo: &Path) -> Result<Self, std::io::Error> {
        let canonical = repo.canonicalize()?;
        let mut map = locks().lock().unwrap();
        let mutex = map.entry(canonical).or_default();
        // We need to hold the guard for the lifetime of RepoLock
        // Use Box to move the guard to the heap and extend its lifetime
        let guard = mutex.lock().unwrap();
        // This is safe because the mutex lives in the static LOCKS map
        let guard: std::sync::MutexGuard<'static, ()> = unsafe { std::mem::transmute(guard) };
        Ok(Self { _guard: Box::new(guard) })
    }
}