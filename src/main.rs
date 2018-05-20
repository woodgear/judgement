extern crate colored;
extern crate walkdir;
extern crate subprocess;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate failure;

use std::env::current_dir;

mod judegment;

use judegment::JudegMent;
use std::path::PathBuf;
use std::path::Component;
use std::ffi::OsString;

fn main() {
	let current_dir = current_dir().unwrap();
	let answer_location = find_answer_dir(current_dir.clone()).unwrap();
	let exam_root_dir = answer_location.parent().unwrap().to_path_buf();
	JudegMent::new(exam_root_dir, answer_location).start_on_dir(current_dir);
}

fn find_answer_dir(cwd: PathBuf) -> Result<PathBuf, FindAnswerErr> {
	if cwd.components().last() == Some(Component::RootDir) {
		return Err(FindAnswerErr::CoudldNotFoundAnswer);
	}

	if cwd.is_dir() {
		let mut answer_name = OsString::from(".");
		answer_name.push(cwd.file_name().ok_or(FindAnswerErr::CoudldNotFoundName(cwd.clone()))?);
		answer_name.push(".answer");
		let expect_answer_location = cwd.join(answer_name);
		if expect_answer_location.exists() && expect_answer_location.is_dir() {
			return Ok(expect_answer_location);
		}
	}
	let parent = cwd.parent()
		.ok_or(FindAnswerErr::CoudldNotFoundParent(cwd.clone()))?;
	return find_answer_dir(parent.to_path_buf());
}

#[derive(Fail, Debug)]
enum FindAnswerErr {
	#[fail(display = "in the root dir could not find the answer location")]
	CoudldNotFoundAnswer,
	#[fail(display = "could not find this path name {:?}", _0)]
	CoudldNotFoundName(PathBuf),
	#[fail(display = "could not find this path parent {:?}", _0)]
	CoudldNotFoundParent(PathBuf),
}


#[cfg(test)]
mod tests {
	use super::*;
	use std::env::current_dir;

	#[test]
	fn test_find_answer_dir() {
		let cwd = current_dir().unwrap();
		let answer_dir = cwd.join("mock_data").join(".mock_data.answer");
		assert_eq!(answer_dir, find_answer_dir(cwd.join("mock_data")).unwrap());
		assert_eq!(answer_dir, find_answer_dir(cwd.join("mock_data/a-1")).unwrap());
	}
}