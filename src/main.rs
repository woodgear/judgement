extern crate colored;
extern crate walkdir;
extern crate subprocess;
#[macro_use]
extern crate derive_new;

use std::env::current_dir;

mod judegment;

use judegment::JudegMent;

fn main() {
	let current_dir = current_dir().unwrap();
	let location = current_dir.clone();
	let answer_location = current_dir.join(".answer");
	JudegMent::new(location, answer_location).start();
}
