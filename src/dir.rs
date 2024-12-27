use std::ops::{ Deref, DerefMut };
use crate::FsPath;



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirRef(pub FsPath);
impl DirRef {

	/* CONSTRUCTOR METHODS */

	/// Create a new dir with an owned path.
	pub fn new(path:&str) -> DirRef {
		DirRef(FsPath::new(path))
	}

	/// Create a new dir with a statically borrowed path.
	pub const fn new_const(path:&'static str) -> DirRef {
		DirRef(FsPath::new_const(path))
	}



	/* PROPERTY GETTER METHODS */

	/// Get the name of the file.
	pub fn dir_name(&self) -> &str {
		self.0.last_node()
	}
}



/* FsPath INHERITED METHODS */
impl Deref for DirRef {
	type Target = FsPath;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for DirRef {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}