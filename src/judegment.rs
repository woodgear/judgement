use std::path::PathBuf;
use std::fs;
use std::process::Command;
use colored::*;
use walkdir::WalkDir;
use std::ffi::OsStr;
use subprocess::Exec;
use subprocess::Redirection;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

pub struct JudegMent {
	examinee: PathBuf,
	examiner: PathBuf,
}


pub struct TempTestHandle {
	paper: Paper,
	examiner: Option<Box<Score>>,
	examinee: Option<Box<Examination>>,
}

#[derive(Clone, Debug)]
struct Paper {
	name: String,
	path: PathBuf,
}

impl Paper {
	fn new(name: String, path: PathBuf) -> Self {
		Self {
			name,
			path,
		}
	}
}

impl JudegMent {
	pub fn new(examinee: PathBuf, examiner: PathBuf) -> Self {
		Self {
			examinee,
			examiner,
		}
	}

	pub fn start(&self) {
		let all_test: Vec<TempTestHandle> = self.get_all_tester();
		let all_count = all_test.len();

		let mut miss_examinee: Vec<Paper> = vec![];
		let mut miss_examiner: Vec<Paper> = vec![];
		let mut success_count = 0;
		let mut fail_count = 0;


		let avaiable_test: Vec<TestHandler> = all_test
			.into_iter()
			.filter_map(|tt| {
				match (tt.examiner, tt.examinee) {
					(Some(er), Some(ee)) => {
						Some(TestHandler {
							paper: tt.paper,
							examinee: ee,
							examiner: er,
						})
					}
					(None, Some(ee)) => {
						miss_examiner.push(tt.paper);
						None
					}
					(Some(er), None) => {
						miss_examinee.push(tt.paper);
						None
					}
					(None, None) => {
						miss_examiner.push(tt.paper.clone());
						miss_examinee.push(tt.paper);
						None
					}
				}
			})
			.collect();

		let avaiable_test_count = avaiable_test.len();

		let _: Vec<()> = avaiable_test
			.into_iter()
			.filter_map(|t| {
				let paper = t.paper.clone();
				match t.start_test() {
					Ok(()) => {
						success_count += 1;
						println!("{}", format!("success: {} pass", paper.name.clone()).green());
						Some(())
					}
					Err(e) => {
						fail_count += 1;
						println!("{}", format!("fail: {} fail {}", paper.name.clone(), e).red());
						None
					}
				}
			})
			.collect();

		if success_count == all_count {
			println!("{}", format!("good job you has finish all test").yellow());
		}
		println!("{}",
		         format!("all_test:{} unfinished:{} miss_examiner:{} miss_examinee:{} success:{} fail:{}",
		                 all_count,
		                 all_count - avaiable_test_count,
		                 miss_examiner.len(),
		                 miss_examinee.len(),
		                 success_count,
		                 fail_count).yellow()
		);
	}

	fn get_all_tester(&self) -> Vec<TempTestHandle> {
		let res: Vec<TempTestHandle> = WalkDir::new(self.examinee.clone())
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|p| !p.path().is_dir())
			.filter(|p| p.path().file_stem() != Some(OsStr::new("README")))
			.filter(|p| p.path().extension() == Some(OsStr::new("md")))
			.filter(|p| !p.path().starts_with("."))
			.map(|p| {
				p.path().to_path_buf()
			})
			.map(|p| {
				//TODO: a lot of unwrap fix it
				//TODO: this find examiner is n^2
				let name = p.file_stem().unwrap().to_str().unwrap().to_string();
				let except_examinee_dir = p.parent().unwrap().to_path_buf();
				let subfix_dir = except_examinee_dir.strip_prefix(&self.examinee).unwrap();
				let except_examiner_dir = self.examiner.clone().join(subfix_dir);
				let examiner = self.find_examiner(&except_examiner_dir, &name);
				let examinee = self.find_examinee(&except_examinee_dir, &name);
				TempTestHandle {
					paper: Paper::new(name, p),
					examinee,
					examiner,
				}
			})
			.collect();
		res
	}

	fn find_examiner(&self, path: &PathBuf, name: &String) -> Option<Box<Score>> {
		let paths: Vec<PathBuf> = WalkDir::new(path)
			.max_depth(1)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|p| !p.path().is_dir())
			.filter(|p| p.path().file_stem() == Some(OsStr::new(name)))
			.map(|p| p.path().to_path_buf())
			.collect();

		let mut examiners: Vec<Box<Score>> = paths
			.into_iter()
			.filter_map(|p| {
				let res = match p.extension().map(|e| e.to_str()) {
					Some(Some("oa")) => Some(Box::new(OAExaminer::new(p)) as Box<Score>),
					Some(Some("qa")) => Some(Box::new(QAExaminer::new(p)) as Box<Score>),
					_ => None
				};
				res
			}).collect();

		examiners.pop()
	}

	fn find_examinee(&self, path: &PathBuf, name: &String) -> Option<Box<Examination>> {
		let paths: Vec<PathBuf> = WalkDir::new(path)
			.max_depth(1)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|p| !p.path().is_dir())
			.filter(|p| p.path().file_stem() == Some(OsStr::new(name)))
			.map(|p| p.path().to_path_buf())
			.collect();
		let mut examinees: Vec<Box<Examination>> = paths
			.into_iter()
			.filter_map(|p| {
				let res = match p.extension().and_then(|e| e.to_str()) {
					Some("sh") => Some(Box::new(ShellExaminee::new(p)) as Box<Examination>),
					_ => None
				};
				res
			}).collect();

		examinees.pop()
	}
}

struct TestHandler {
	paper: Paper,
	examiner: Box<Score>,
	examinee: Box<Examination>,
}

impl TestHandler {
	fn start_test(&self) -> Result<(), String> {
		self.examiner.score(&self.examinee)
	}
}


trait Examination {
	fn eval(&self, question: String) -> Result<String, String>;
}

trait Score {
	fn score(&self, e: &Box<Examination>) -> Result<(), String>;
}

#[derive(new)]
struct ShellExaminee {
	path: PathBuf
}

fn eval_cmd(cmd: String) -> Result<String, String> {
	Exec::shell(cmd)
		.stdout(Redirection::Pipe)
		.capture()
		.map_err(|e| e.to_string())
		.and_then(|r| {
			if r.exit_status.success() {
				Ok(r.stdout_str())
			} else {
				Err(r.stderr_str())
			}
		})
}


impl Examination for ShellExaminee {
	fn eval(&self, question: String) -> Result<String, String> {
		let path = self.path.to_str()
			.ok_or(format!("path to sting fail"))?;
		let cmd = format!("sh  {} {}", path, question);
		let res = eval_cmd(cmd);
		res
	}
}

impl Score for OAExaminer {
	fn score(&self, e: &Box<Examination>) -> Result<(), String> {
		let res = e.eval("".into())?;
		let answer = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
		if answer.trim() == res.trim() {
			return Ok(());
		}
		Err("".into())
	}
}

impl Score for QAExaminer {
	fn score(&self, e: &Box<Examination>) -> Result<(), String> {
		let file = File::open(self.path.clone()).unwrap();
		let file1 = File::open(self.path.clone()).unwrap();

		let mut lines = BufReader::new(file).lines();
		match (lines.next(), lines.next()) {
			(Some(Ok(input)), Some(Ok(answer))) => {
				let res = e.eval(input.clone())?;
				if answer.trim() != res.trim() {
					return Err(format!("given {} except {} find {}", input, answer, res));
				}
			}
			_ => {}
		};
		Ok(())
	}
}

#[derive(new)]
struct OAExaminer {
	path: PathBuf
}

#[derive(new)]
struct QAExaminer {
	path: PathBuf
}


struct Tester {
	path: PathBuf
}


#[cfg(test)]
mod tests {
	use super::*;
	use std::env::current_dir;

	#[test]
	fn test_get_all_handle() {
		let current_dir = current_dir().unwrap().join("mock_data");
		let location = current_dir.clone();
		let answer_location = current_dir.join(".answer");

		let judge = JudegMent::new(current_dir, answer_location);
		let tests = judge.get_all_tester();
		assert_eq!(tests.len(), 2);

		for t in tests {
			println!("{:?}", t.paper);
			assert!(t.examiner.is_some());
			assert!(t.examinee.is_some());
		}
	}

	#[test]
	fn test_find_examiner() {
		let judge = JudegMent::new(PathBuf::new(), PathBuf::new());
		let res = judge.find_examiner(&current_dir().unwrap().join("mock_data/.answer/a-1"), &"1".into());
		assert!(res.is_some())
	}

	#[test]
	fn test_judgement() {
		let current_dir = current_dir().unwrap().join("mock_data");
		let location = current_dir.clone();
		let answer_location = current_dir.join(".answer");

		let judge = JudegMent::new(current_dir, answer_location);
		judge.start();
	}
}
