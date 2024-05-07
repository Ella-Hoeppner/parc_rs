use std::sync::atomic::AtomicUsize;

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
