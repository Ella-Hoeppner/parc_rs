use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{self, Ordering};

use crate::darc::Darc;
use crate::potentially_atomic_usize::PotentiallyAtomicCounter;

pub struct ParcInner<T> {
  pub(crate) rc: RefCell<PotentiallyAtomicCounter>,
  pub(crate) data: T,
}

impl<T> ParcInner<T> {
  pub fn new(data: T) -> Self {
    Self {
      rc: PotentiallyAtomicCounter::new_nonatomic(1).into(),
      data,
    }
  }
  pub fn rc(&mut self) -> u32 {
    self.rc.borrow_mut().copy_u32()
  }
  pub fn force_atomic(&mut self) {
    if !self.is_atomic() {
      self
        .rc
        .replace_with(|rc| PotentiallyAtomicCounter::new_atomic(rc.copy_u32()));
    }
  }
  pub fn is_atomic(&self) -> bool {
    self.rc.borrow().is_atomic()
  }
}

// Potentially Atomic Reference Counter
pub struct Parc<T> {
  pub(crate) inner: NonNull<ParcInner<T>>,
  phantom: PhantomData<ParcInner<T>>,
}

impl<T> Parc<T> {
  pub fn new(data: T) -> Self {
    let boxed = Box::new(ParcInner::new(data));
    Self {
      inner: NonNull::new(Box::into_raw(boxed)).unwrap(),
      phantom: PhantomData,
    }
  }
  pub fn rc(&mut self) -> u32 {
    unsafe { self.inner.as_mut() }.rc()
  }
  pub fn is_atomic(&mut self) -> bool {
    unsafe { self.inner.as_ref() }.is_atomic()
  }
}

impl<T> Deref for Parc<T> {
  type Target = T;

  fn deref(&self) -> &T {
    let inner = unsafe { self.inner.as_ref() };
    &inner.data
  }
}

impl<T> Clone for Parc<T> {
  fn clone(&self) -> Self {
    let inner = unsafe { self.inner.as_ref() };
    RefMut::map(inner.rc.borrow_mut(), |parc| {
      match parc {
        PotentiallyAtomicCounter::Atomic(arc) => {
          let old_rc = arc.fetch_add(1, Ordering::Relaxed);
          if old_rc >= i32::MAX as u32 {
            std::process::abort();
          }
        }
        PotentiallyAtomicCounter::NonAtomic(rc) => {
          if *rc >= i32::MAX as u32 {
            std::process::abort();
          }
          *rc += 1;
        }
      }
      parc
    });
    Self {
      inner: self.inner,
      phantom: PhantomData,
    }
  }
}

impl<T> Drop for Parc<T> {
  fn drop(&mut self) {
    let inner = unsafe { self.inner.as_mut() };
    match inner.rc.get_mut() {
      PotentiallyAtomicCounter::Atomic(rc) => {
        if rc.fetch_sub(1, Ordering::Release) != 1 {
          return;
        }
        atomic::fence(Ordering::Acquire);
        drop(unsafe { Box::from_raw(inner) });
      }
      PotentiallyAtomicCounter::NonAtomic(rc) => {
        *rc -= 1;
        if *rc == 0 {
          drop(unsafe { Box::from_raw(inner) });
        }
      }
    }
  }
}

impl<T> From<Darc<T>> for Parc<T> {
  fn from(mut darc: Darc<T>) -> Self {
    unsafe { darc.inner.as_mut() }.force_atomic();
    let parc = Parc {
      inner: darc.inner,
      phantom: PhantomData,
    };
    std::mem::forget(darc);
    parc
  }
}
