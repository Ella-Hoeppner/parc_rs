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

  #[test]
  fn darc_rc_tracks_across_threads() {
    let mut parc = Parc::new("test");
    assert_eq!(parc.rc(), 1);
    assert!(!parc.is_atomic());
    let mut darc = Darc::from(parc.clone());
    let handle = std::thread::spawn(move || {
      assert_eq!(darc.rc(), 2);
    });
    assert_eq!(parc.rc(), 2);
    handle.join().unwrap();
    assert_eq!(parc.rc(), 1);
  }

  #[test]
  fn darc_to_parc() {
    let mut parc = Parc::new("test");
    let darc = Darc::from(parc.clone());
    let handle = std::thread::spawn(move || {
      let mut inner_parc = Parc::from(darc);
      assert_eq!(inner_parc.rc(), 2);
    });
    assert_eq!(parc.rc(), 2);
    handle.join().unwrap();
    assert_eq!(parc.rc(), 1);
  }
}
