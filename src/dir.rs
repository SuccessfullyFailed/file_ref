use std::{ error::Error, ops::{ Deref, DerefMut } };
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

	/// Check if the files exists.
	pub fn exists(&self) -> bool {
		std::path::Path::new(&self.path()).exists() && std::fs::metadata(&self.path()).map(|data| data.is_dir()).unwrap_or(false)
	}



	/* DIRECTORY WRITING METHODS */

	/// Create the directory.
	pub fn create(&self) -> Result<(), Box<dyn Error>> {
		if self.exists() {
			Err(format!("Could not create dir \"{}\". Dir already exists.", self.path()).into())
		} else {
			std::fs::create_dir(self.path()).map_err(|error| error.into())
		}
	}



	/* DIRECTORY REMOVING METHODS */

	/// Delete the directory.
	pub fn delete(&self) -> Result<(), Box<dyn Error>> {
		std::fs::remove_dir_all(self.path()).map_err(|error| error.into())
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