use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicUsize, Ordering};

pub struct Arc<T> {
  ptr: NonNull<ArcInner<T>>,
  phantom: PhantomData<ArcInner<T>>,
}

pub struct ArcInner<T> {
  rc: AtomicUsize,
  data: T,
}

impl<T> Arc<T> {
  pub fn new(data: T) -> Arc<T> {
    let boxed = Box::new(ArcInner {
      rc: AtomicUsize::new(1),
      data,
    });
    Arc {
      ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
      phantom: PhantomData,
    }
  }
  pub fn rc(&self) -> &AtomicUsize {
    &(unsafe { self.ptr.as_ref() }).rc
  }
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}

impl<T> Deref for Arc<T> {
  type Target = T;

  fn deref(&self) -> &T {
    let inner = unsafe { self.ptr.as_ref() };
    &inner.data
  }
}

impl<T> Clone for Arc<T> {
  fn clone(&self) -> Arc<T> {
    let inner = unsafe { self.ptr.as_ref() };
    let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);

    if old_rc >= isize::MAX as usize {
      std::process::abort();
    }

    Self {
      ptr: self.ptr,
      phantom: PhantomData,
    }
  }
}

impl<T> Drop for Arc<T> {
  fn drop(&mut self) {
    let inner = unsafe { self.ptr.as_ref() };
    if inner.rc.fetch_sub(1, Ordering::Release) != 1 {
      return;
    }
    atomic::fence(Ordering::Acquire);
    drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
  }
}
