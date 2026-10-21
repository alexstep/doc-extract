use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub fn read<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
  lock.read().unwrap_or_else(|err| err.into_inner())
}

pub fn write<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
  lock.write().unwrap_or_else(|err| err.into_inner())
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, RwLock};
  use std::thread;

  use super::*;

  #[test]
  fn poisoned_lock_returns_the_guard() {
    let lock = Arc::new(RwLock::new(7_u32));
    let worker = Arc::clone(&lock);
    let _ = thread::spawn(move || {
      let _guard = worker.write().unwrap();
      panic!("poison the lock");
    })
    .join();

    assert!(lock.read().is_err());
    assert_eq!(*read(&lock), 7);
    *write(&lock) = 8;
    assert_eq!(*read(&lock), 8);
  }
}
