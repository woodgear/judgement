use std::path::PathBuf;
use std::fs;
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
	pub fn start_on_dir(&self, cwd: PathBuf) {
		let all_test: Vec<TempTestHandle> = self.get_all_tester();
		let all_count = all_test.len();

		let mut miss_examinee: Vec<Paper> = vec![];
		let mut miss_examiner: Vec<Paper> = vec![];

		let avaiable_test = self.get_avaiable_test(all_test, &mut miss_examinee, &mut miss_examiner);
		let miss_examiner_count = miss_examiner.len();

		let target_test = self.get_target_test(avaiable_test, cwd);
		let target_test_count = target_test.len();

		let success_count = self.start_test(target_test);
		let fail_count = target_test_count - success_count;
		if success_count == all_count {
			println!("{}", format!("good job you has finish all test").blue());
		}
		println!("{}",
		         format!("all_test:{} unfinished:{} success:{} fail:{} miss_examiner:{}",
		                 all_count,
		                 all_count - success_count,
		                 success_count,
		                 fail_count,
		                 miss_examiner_count,
		         ).yellow()
		);
	}
	fn get_target_test(&self, list: Vec<TestHandler>, target_dir: PathBuf) -> Vec<TestHandler> {
		list.into_iter()
			.filter(|t| {
				t.paper.path.starts_with(&target_dir)
			})
			.collect()
	}
	fn get_avaiable_test(&self, list: Vec<TempTestHandle>, miss_examinee: &mut Vec<Paper>, miss_examiner: &mut Vec<Paper>) -> Vec<TestHandler> {
		let avaiable_test: Vec<TestHandler> = list
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
					(None, Some(_)) => {
						miss_examiner.push(tt.paper);
						None
					}
					(Some(_), None) => {
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
		avaiable_test
	}

	fn start_test(&self, list: Vec<TestHandler>) -> usize {
		let success: Vec<()> = list
			.into_iter()
			.filter_map(|t| {
				let paper = t.paper.clone();
				let name: String = paper.path.strip_prefix(&self.examinee)
					.map(|p| p.to_string_lossy().to_string())
					.unwrap_or(paper.name);

				match t.start_test() {
					Ok(()) => {
						println!("{}", format!("success: {}", name).green());
						Some(())
					}
					Err(e) => {
						println!("{}", format!("fail: {} {}", name, e).red());
						None
					}
				}
			})
			.collect();
		return success.len();
	}


	fn get_all_tester(&self) -> Vec<TempTestHandle> {
		let res: Vec<TempTestHandle> = WalkDir::new(self.examinee.clone())
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|p| !p.path().is_dir())
			.filter(|p| !p.path().starts_with("."))
			.filter(|p| p.path().file_stem() != Some(OsStr::new("README")))
			.filter(|p| p.path().extension() == Some(OsStr::new("md")))
			.map(|p| {
				p.path().to_path_buf()
			})
			.map(|p| {
				//TODO: a lot of unwrap fix it
				//TODO: this find examiner is n^2
				let name = p.file_stem().unwrap().to_str().unwrap().to_string();
				let expect_examinee_dir = p.parent().unwrap().to_path_buf();
				let subfix_dir = expect_examinee_dir.strip_prefix(&self.examinee).unwrap();
				let expect_examiner_dir = self.examiner.clone().join(subfix_dir);
				let examiner = self.find_examiner(&expect_examiner_dir, &name);
				let examinee = self.find_examinee(&expect_examinee_dir, &name);

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


#[cfg(test)]
mod tests {
	use super::*;
	use std::env::current_dir;

	#[test]
	fn test_get_all_handle() {
		let current_dir = current_dir().unwrap().join("mock_data");
		let answer_location = current_dir.join(".mock_data.answer");

		let judge = JudegMent::new(current_dir, answer_location);
		let tests = judge.get_all_tester();
		assert_eq!(tests.len(), 6);
		let mut me = vec![];
		let mut mr = vec![];
		let tests = judge.get_avaiable_test(tests, &mut me, &mut mr);
		assert_eq!(tests.len(), 0);
		assert_eq!(me.len(), 6);
		assert_eq!(mr.len(), 4);
	}

	#[test]
	fn test_find_examiner() {
		let judge = JudegMent::new(PathBuf::new(), PathBuf::new());
		let res = judge.find_examiner(&current_dir().unwrap().join("mock_data/.mock_data.answer/a-1"), &"1".into());
		assert!(res.is_some())
	}

	#[test]
	fn test_judgement() {
		let current_dir = current_dir().unwrap().join("mock_data");
		let answer_location = current_dir.join(".answer");

		let judge = JudegMent::new(current_dir.clone(), answer_location);
		judge.start_on_dir(current_dir);
	}
}
