use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{self, Ordering};

use crate::parc::{Parc, ParcInner};
use crate::potentially_atomic_usize::PotentiallyAtomicUsize;

pub struct Darc<T> {
  inner: NonNull<ParcInner<T>>,
  phantom: PhantomData<ParcInner<T>>,
}

impl<T> Darc<T> {
  pub fn rc(&mut self) -> usize {
    unsafe { self.inner.as_mut() }.rc()
  }
}

impl<T> Deref for Darc<T> {
  type Target = T;

  fn deref(&self) -> &T {
    let inner = unsafe { self.inner.as_ref() };
    &inner.data
  }
}

impl<T> Clone for Darc<T> {
  fn clone(&self) -> Self {
    let inner = unsafe { self.inner.as_ref() };
    RefMut::map(inner.rc.borrow_mut(), |darc| {
      match darc {
        PotentiallyAtomicUsize::Atomic(arc) => {
          let old_rc = arc.fetch_add(1, Ordering::Relaxed);
          if old_rc >= isize::MAX as usize {
            std::process::abort();
          }
        }
        PotentiallyAtomicUsize::NonAtomic(_) => {
          unreachable!()
        }
      }
      darc
    });
    Self {
      inner: self.inner,
      phantom: PhantomData,
    }
  }
}

unsafe impl<T> Send for Darc<T> {}
unsafe impl<T> Sync for Darc<T> {}

impl<T> Drop for Darc<T> {
  fn drop(&mut self) {
    let inner = unsafe { self.inner.as_mut() };
    match inner.rc.get_mut() {
      PotentiallyAtomicUsize::Atomic(rc) => {
        if rc.fetch_sub(1, Ordering::Release) != 1 {
          return;
        }
        atomic::fence(Ordering::Acquire);
        drop(unsafe { Box::from_raw(inner) });
      }
      PotentiallyAtomicUsize::NonAtomic(_) => {
        unreachable!()
      }
    }
  }
}

impl<T> From<Parc<T>> for Darc<T> {
  fn from(mut parc: Parc<T>) -> Self {
    unsafe { parc.inner.as_mut() }.force_atomic();
    let darc = Darc {
      inner: parc.inner,
      phantom: PhantomData,
    };
    std::mem::forget(parc);
    darc
  }
}
