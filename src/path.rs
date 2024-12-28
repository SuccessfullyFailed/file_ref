use std::{error::Error, ops::Add};
use crate::DirRef;



// Could be chars, but will be used as str's mainly, so this stops the program from converting.
pub(crate) const SEPARATOR:&str = "/";
const INVALID_SEPARATOR:&str = "\\";



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FsPath {
	StaticStr(&'static str),
	Owned(String)
}
impl FsPath {

	/* CONSTRUCTOR METHODS */

	/// Create a new owned path.
	pub fn new(path:&str) -> FsPath {
		FsPath::Owned(path.replace(INVALID_SEPARATOR, SEPARATOR))
	}

	/// Create a new statically borrowed path.
	pub const fn new_const(path:&'static str) -> FsPath {
		FsPath::StaticStr(path)
	}



	/* PROPERTY GETTER METHODS */

	/// Get the raw path.
	pub fn path(&self) -> &str {
		match self {
			FsPath::StaticStr(path) => *path,
			FsPath::Owned(path) => path.as_str()
		}
	}

	/// Get the directory the file is in.
	pub fn parent_dir(&self) -> Result<DirRef, Box<dyn Error>> {
		let path:&str = self.path();
		let nodes:Vec<&str> = self.path_nodes();
		if nodes.len() <= 1 {
			Err(format!("Could not get dir of file \"{path}\", as it only contains the file name.").into())
		} else {
			let parent_dir_len:usize = nodes[..nodes.len() - 1].join(SEPARATOR).len();
			Ok(DirRef::new(&path[..parent_dir_len]))
		}
	}

	/// Get a list of nodes in the path.
	pub(crate) fn path_nodes(&self) -> Vec<&str> {
		self.path().split(SEPARATOR).collect()
	}

	/// Get the last node of the path.
	pub(crate) fn last_node(&self) -> &str {
		self.path().split(SEPARATOR).last().unwrap_or_default()
	}



	/* OPERATION METHODS */

	/// If the parent dir does not exist, create it.
	pub fn guarantee_parent_dir(&self) -> Result<(), Box<dyn Error>> {
		let parent_dir:DirRef = self.parent_dir()?;
		if !parent_dir.exists() {
			parent_dir.create()?;
		}
		Ok(())
	}
}
impl Add<&str> for FsPath {
	type Output = FsPath;

	fn add(self, addition:&str) -> Self::Output {
		FsPath::new(&(self.path().to_owned() + addition))
	}
}



/* STR INHERITED METHODS */
macro_rules! impl_inherit_str {

	// Case for methods without arguments.
	($fn_name:ident, $output_type:ty) => {
		impl FsPath {
			pub fn $fn_name(&self) -> $output_type {
				self.path().$fn_name()
			}
		}
	};

	// Case for methods with arguments.
	($fn_name:ident, $output_type:ty, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FsPath {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> $output_type {
				self.path().$fn_name($($arg_name),*)
			}
		}
	};

	// Case for methods returning `FsPath`.
	(ret_self $fn_name:ident) => {
		impl FsPath {
			pub fn $fn_name(&self) -> FsPath {
				FsPath::new(&self.path().$fn_name())
			}
		}
	};

	// Case for methods returning `FsPath` with arguments.
	(ret_self $fn_name:ident, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FsPath {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> FsPath {
				FsPath::new(&self.path().$fn_name($($arg_name),*))
			}
		}
	};

	// Case for methods returning `Option<FsPath>`.
	(ret_self_opt $fn_name:ident) => {
		impl FsPath {
			pub fn $fn_name(&self) -> Option<FsPath> {
				self.path().$fn_name().map(|path| FsPath::new(path))
			}
		}
	};

	// Case for methods returning `Option<FsPath>` with arguments.
	(ret_self_opt $fn_name:ident, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FsPath {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> Option<FsPath> {
				self.path().$fn_name($($arg_name),*).map(|path| FsPath::new(path))
			}
		}
	};
}
impl_inherit_str!(len, usize);
impl_inherit_str!(is_empty, bool);
impl_inherit_str!(is_char_boundary, bool, (index:usize));
impl_inherit_str!(contains, bool, (pattern:&str));
impl_inherit_str!(starts_with, bool, (prefix:&str));
impl_inherit_str!(ends_with, bool, (suffix:&str));
impl_inherit_str!(find, Option<usize>, (needle:&str));
impl_inherit_str!(rfind, Option<usize>, (needle:&str));
impl_inherit_str!(split_at, (&str, &str), (mid:usize));
impl_inherit_str!(chars, std::str::Chars<'_>);
impl_inherit_str!(char_indices, std::str::CharIndices<'_>);
impl_inherit_str!(bytes, std::str::Bytes<'_>);
impl_inherit_str!(lines, std::str::Lines<'_>);
impl_inherit_str!(split_whitespace, std::str::SplitWhitespace<'_>);
impl_inherit_str!(split, std::str::Split<'_, char>, (sep:char));
impl_inherit_str!(escape_debug, std::str::EscapeDebug<'_>);
impl_inherit_str!(escape_default, std::str::EscapeDefault<'_>);
impl_inherit_str!(escape_unicode, std::str::EscapeUnicode<'_>);
impl_inherit_str!(splitn, std::str::SplitN<'_, char>, (n:usize, sep:char));
impl_inherit_str!(rsplitn, std::str::RSplitN<'_, char>, (n:usize, sep:char));
impl_inherit_str!(ret_self to_lowercase);
impl_inherit_str!(ret_self to_uppercase);
impl_inherit_str!(ret_self trim);
impl_inherit_str!(ret_self trim_start);
impl_inherit_str!(ret_self trim_end);
impl_inherit_str!(ret_self repeat, (n:usize));
impl_inherit_str!(ret_self replace, (from:&str, to:&str));
impl_inherit_str!(ret_self_opt strip_prefix, (prefix:&str));
impl_inherit_str!(ret_self_opt strip_suffix, (suffix:&str));



#[cfg(test)]
mod tests {
	use super::*;



	#[test]
	fn test_new() {
		let path: &str = "dir\\file.txt";
		let fs_path: FsPath = FsPath::new(path);
		assert_eq!(fs_path.path(), "dir/file.txt");
	}

	#[test]
	fn test_new_const() {
		const PATH: &str = "static/dir/file.txt";
		let fs_path: FsPath = FsPath::new_const(PATH);
		assert_eq!(fs_path.path(), PATH);
	}

	#[test]
	fn test_path() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		assert_eq!(fs_path.path(), "dir/file.txt");
	}

	#[test]
	fn test_parent_dir() {
		let fs_path: FsPath = FsPath::new("dir/subdir/file.txt");
		let parent: DirRef = fs_path.parent_dir().unwrap();
		assert_eq!(parent.path(), "dir/subdir");
	}

	#[test]
	fn test_parent_dir_root() {
		let fs_path: FsPath = FsPath::new("file.txt");
		assert!(fs_path.parent_dir().is_err());
	}

	#[test]
	fn test_path_nodes() {
		let fs_path: FsPath = FsPath::new("dir/subdir/file.txt");
		let nodes: Vec<&str> = fs_path.path_nodes();
		assert_eq!(nodes, vec!["dir", "subdir", "file.txt"]);
	}

	#[test]
	fn test_last_node() {
		let fs_path: FsPath = FsPath::new("dir/subdir/file.txt");
		assert_eq!(fs_path.last_node(), "file.txt");
	}

	#[test]
	fn test_len() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		assert_eq!(fs_path.len(), 12);
	}

	#[test]
	fn test_is_empty() {
		let fs_path: FsPath = FsPath::new("");
		assert!(fs_path.is_empty());

		let fs_path: FsPath = FsPath::new("not_empty");
		assert!(!fs_path.is_empty());
	}

	#[test]
	fn test_contains() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		assert!(fs_path.contains("file"));
		assert!(!fs_path.contains("no_file"));
	}

	#[test]
	fn test_starts_with() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		assert!(fs_path.starts_with("dir"));
		assert!(!fs_path.starts_with("file"));
	}

	#[test]
	fn test_ends_with() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		assert!(fs_path.ends_with("file.txt"));
		assert!(!fs_path.ends_with("dir"));
	}

	#[test]
	fn test_to_lowercase() {
		let fs_path: FsPath = FsPath::new("DIR/FILE.TXT");
		let lower: FsPath = fs_path.to_lowercase();
		assert_eq!(lower.path(), "dir/file.txt");
	}

	#[test]
	fn test_to_uppercase() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		let upper: FsPath = fs_path.to_uppercase();
		assert_eq!(upper.path(), "DIR/FILE.TXT");
	}

	#[test]
	fn test_trim() {
		let fs_path: FsPath = FsPath::new("   dir/file.txt   ");
		let trimmed: FsPath = fs_path.trim();
		assert_eq!(trimmed.path(), "dir/file.txt");
	}

	#[test]
	fn test_strip_prefix() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		let stripped: Option<FsPath> = fs_path.strip_prefix("dir/");
		assert!(stripped.is_some());
		assert_eq!(stripped.unwrap().path(), "file.txt");
	}

	#[test]
	fn test_strip_suffix() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		let stripped: Option<FsPath> = fs_path.strip_suffix(".txt");
		assert!(stripped.is_some());
		assert_eq!(stripped.unwrap().path(), "dir/file");
	}

	#[test]
	fn test_replace() {
		let fs_path: FsPath = FsPath::new("dir/file.txt");
		let replaced: FsPath = fs_path.replace("file", "document");
		assert_eq!(replaced.path(), "dir/document.txt");
	}

	#[test]
	fn test_repeat() {
		let fs_path: FsPath = FsPath::new("file_");
		let repeated: FsPath = fs_path.repeat(3);
		assert_eq!(repeated.path(), "file_file_file_");
	}
}
