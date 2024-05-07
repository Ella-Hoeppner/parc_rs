mod arc;
pub mod parc;

#[cfg(test)]
mod test {
  use crate::parc::Parc;
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
}
