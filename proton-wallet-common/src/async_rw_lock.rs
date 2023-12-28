use async_std::task;
use std::{
    sync::{Arc, Mutex, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct AsyncRwLock<T> {
    inner: Arc<RwLock<T>>,
    write_lock_held: Arc<Mutex<bool>>,
}

impl<T> AsyncRwLock<T> {
    pub fn new(data: T) -> Self {
        AsyncRwLock {
            inner: Arc::new(RwLock::new(data)),
            write_lock_held: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn read(&self) -> Result<RwLockReadGuard<'_, T>, PoisonError<RwLockReadGuard<'_, T>>> {
        while *self.write_lock_held.lock().unwrap() {
            task::sleep(Duration::from_millis(100)).await;
        }

        self.inner.read()
    }

    pub async fn write(&self) -> Result<RwLockWriteGuard<'_, T>, PoisonError<RwLockWriteGuard<'_, T>>> {
        while *self.write_lock_held.lock().unwrap() {
            task::sleep(Duration::from_millis(100)).await;
        }

        *self.write_lock_held.lock().unwrap() = true;
        self.inner.write()
    }

    pub fn release_write_lock(&self) {
        let mut write_lock_held = self.write_lock_held.lock().unwrap();
        *write_lock_held = false;
    }
}
