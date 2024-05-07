use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicUsize, Ordering};

#[derive(Debug)]
pub enum PotentiallyAtomicUsize {
  Atomic(AtomicUsize),
  NonAtomic(usize),
}
impl PotentiallyAtomicUsize {
  pub fn new_nonatomic(x: usize) -> Self {
    Self::NonAtomic(x)
  }
  pub fn new_atomic<T: Into<AtomicUsize>>(x: T) -> Self {
    Self::Atomic(x.into())
  }
  pub fn copy_usize(&mut self) -> usize {
    match self {
      PotentiallyAtomicUsize::Atomic(atomic) => *atomic.get_mut(),
      PotentiallyAtomicUsize::NonAtomic(nonatomic) => *nonatomic,
    }
  }
  pub fn is_atomic(&self) -> bool {
    match self {
      PotentiallyAtomicUsize::Atomic(_) => true,
      PotentiallyAtomicUsize::NonAtomic(_) => false,
    }
  }
}
impl From<PotentiallyAtomicUsize> for AtomicUsize {
  fn from(value: PotentiallyAtomicUsize) -> Self {
    match value {
      PotentiallyAtomicUsize::Atomic(x) => x,
      PotentiallyAtomicUsize::NonAtomic(x) => x.into(),
    }
  }
}

pub struct Parc<T> {
  ptr: NonNull<ParcInner<T>>,
  phantom: PhantomData<ParcInner<T>>,
}

pub struct ParcInner<T> {
  rc: RefCell<PotentiallyAtomicUsize>,
  data: T,
}

impl<T> Parc<T> {
  pub fn new(data: T) -> Parc<T> {
    let boxed = Box::new(ParcInner {
      rc: PotentiallyAtomicUsize::new_nonatomic(1).into(),
      data,
    });
    Parc {
      ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
      phantom: PhantomData,
    }
  }
  pub fn rc(&mut self) -> usize {
    unsafe { self.ptr.as_mut() }.rc.get_mut().copy_usize()
  }
}

impl<T> Deref for Parc<T> {
  type Target = T;

  fn deref(&self) -> &T {
    let inner = unsafe { self.ptr.as_ref() };
    &inner.data
  }
}

impl<T> Clone for Parc<T> {
  fn clone(&self) -> Parc<T> {
    let inner = unsafe { self.ptr.as_ref() };
    RefMut::map(inner.rc.borrow_mut(), |parc| {
      match parc {
        PotentiallyAtomicUsize::Atomic(arc) => {
          let old_rc = arc.fetch_add(1, Ordering::Relaxed);
          if old_rc >= isize::MAX as usize {
            std::process::abort();
          }
        }
        PotentiallyAtomicUsize::NonAtomic(rc) => {
          if *rc >= isize::MAX as usize {
            std::process::abort();
          }
          *rc += 1;
        }
      }
      parc
    });
    Self {
      ptr: self.ptr,
      phantom: PhantomData,
    }
  }
}

impl<T> Drop for Parc<T> {
  fn drop(&mut self) {
    let inner = unsafe { self.ptr.as_mut() };
    match inner.rc.get_mut() {
      PotentiallyAtomicUsize::Atomic(rc) => {
        if rc.fetch_sub(1, Ordering::Release) != 1 {
          return;
        }
        atomic::fence(Ordering::Acquire);
        drop(unsafe { Box::from_raw(inner) });
      }
      PotentiallyAtomicUsize::NonAtomic(rc) => {
        *rc -= 1;
        if *rc == 0 {
          drop(unsafe { Box::from_raw(inner) });
        }
      }
    }
  }
}
