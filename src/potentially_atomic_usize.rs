use std::sync::atomic::AtomicU32;

#[derive(Debug)]
pub enum PotentiallyAtomicCounter {
  Atomic(AtomicU32),
  NonAtomic(u32),
}
impl PotentiallyAtomicCounter {
  pub fn new_nonatomic(x: u32) -> Self {
    Self::NonAtomic(x)
  }
  pub fn new_atomic<T: Into<AtomicU32>>(x: T) -> Self {
    Self::Atomic(x.into())
  }
  pub fn copy_u32(&mut self) -> u32 {
    match self {
      PotentiallyAtomicCounter::Atomic(atomic) => *atomic.get_mut(),
      PotentiallyAtomicCounter::NonAtomic(nonatomic) => *nonatomic,
    }
  }
  pub fn is_atomic(&self) -> bool {
    match self {
      PotentiallyAtomicCounter::Atomic(_) => true,
      PotentiallyAtomicCounter::NonAtomic(_) => false,
    }
  }
}
impl From<PotentiallyAtomicCounter> for AtomicU32 {
  fn from(value: PotentiallyAtomicCounter) -> Self {
    match value {
      PotentiallyAtomicCounter::Atomic(x) => x,
      PotentiallyAtomicCounter::NonAtomic(x) => x.into(),
    }
  }
}
