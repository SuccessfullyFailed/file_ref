use std::ops::{Deref, DerefMut};
use crate::FsPath;



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileRef(pub FsPath);
impl FileRef {

	/* CONSTRUCTOR METHODS */

	/// Create a new file with an owned path.
	pub fn new(path:&str) -> FileRef {
		FileRef(FsPath::new(path))
	}

	/// Create a new file with a statically borrowed path.
	pub const fn new_const(path:&'static str) -> FileRef {
		FileRef(FsPath::new_const(path))
	}



	/* PROPERTY GETTER METHODS */

	/// Get the name of the file.
	pub fn file_name(&self) -> &str {
		self.0.last_node()
	}

	/// Get the name of the file without extension.
	pub fn file_name_no_extension(&self) -> &str {
		self.file_name().trim_end_matches(self.extension())
	}

	/// Get the extension of the file.
	pub fn extension(&self) -> &str {
		self.file_name().split('.').last().unwrap_or_default()
	}
}



/* FsPath INHERITED METHODS */
impl Deref for FileRef {
	type Target = FsPath;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for FileRef {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}