mod arc;
pub mod darc;
pub mod parc;
pub mod potentially_atomic_usize;

#[cfg(test)]
mod test {
  use crate::{darc::Darc, parc::Parc};

  #[test]
  fn parc_clone_rc_tracks() {
    let mut parc_original = Parc::new("test");
    assert_eq!(parc_original.rc(), 1);
    {
      let mut parc_clone = parc_original.clone();
      assert_eq!(parc_original.rc(), 2);
      assert_eq!(parc_clone.rc(), 2);
    }
    assert_eq!(parc_original.rc(), 1);
  }

  #[test]
  fn darc_rc_tracks() {
    let mut parc = Parc::new("test");
    assert_eq!(parc.rc(), 1);
    assert!(!parc.is_atomic());
    {
      let mut darc = Darc::from(parc.clone());
      assert_eq!(parc.rc(), 2);
      assert_eq!(darc.rc(), 2);
      assert!(parc.is_atomic());
    }
    assert_eq!(parc.rc(), 1);
    assert!(parc.is_atomic());
  }
}
