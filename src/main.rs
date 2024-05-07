#![feature(negative_impls)]

mod arc;
mod parc;

use arc::Arc;
use parc::Parc;

use std::thread::spawn;

fn proof_arc_threadsafe() {
  let arc = Arc::new("test");
  let arc_clone = Arc::clone(&arc);
  let handle = spawn(move || {
    println!("{}", *arc_clone);
  });
  println!("{}", *arc);
  println!("reference count before join: {:?}", arc.rc());
  handle.join().unwrap();
  println!("reference count after join: {:?}", arc.rc());
}

/*fn proof_parc_not_threadsafe() {
  let parc = Parc::new("test");
  let parc_clone = Parc::clone(&parc);
  let handle = spawn(move || {println!("{}", *parc_clone);});
  println!("{}",*parc);
  println!("reference count before join: {:?}", parc.rc());
  handle.join().unwrap();
  println!("reference count after join: {:?}", parc.rc());
}*/

fn main() {
  proof_arc_threadsafe();
}
