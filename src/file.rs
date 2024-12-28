use std::{ error::Error, ops::{ Deref, DerefMut } };
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

	/// Check if self is a dir.
	pub fn is_dir(&self) -> bool {
		self.extension().map(|extension| extension.is_empty()).unwrap_or(true)
	}

	/// Check if self is a file.
	pub fn is_file(&self) -> bool {
		!self.is_dir()
	}

	/// Get the name of the file/dir.
	pub fn name(&self) -> &str {
		self.0.last_node()
	}

	/// Get the name of the file without extension.
	pub fn file_name_no_extension(&self) -> &str {
		self.name().trim_end_matches(&self.extension().map(|extension| (".".to_owned() + extension)).unwrap_or_default())
	}

	/// Get the extension of the file.
	pub fn extension(&self) -> Option<&str> {
		let file_name:&str = self.name();
		if file_name.contains('.') {
			file_name.split('.').last()
		} else {
			None
		}
	}

	/// Check if the files exists.
	pub fn exists(&self) -> bool {
		std::path::Path::new(&self.path()).exists() && std::fs::metadata(&self.path()).is_ok()
	}
	
	/// Check if the file can be accessed.
	pub fn is_accessible(&self) -> bool {
		if self.is_dir() { true } else { std::fs::File::open(&self.path()).is_ok() }
	}



	/* FILE READING METHODS */

	/// Read the contents of the file as a string.
	pub fn read(&self) -> Result<String, Box<dyn Error>> {
		use std::{ fs::File, io::Read };
		
		if self.is_dir() {
			Err(format!("Could not read dir \"{}\". Only able to read files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = File::open(self.path())?;
			let mut contents:String = String::new();
			file.read_to_string(&mut contents)?;
			Ok(contents)
		}
	}

	/// Read the contents of the file as bytes.
	pub fn read_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		use std::{ fs::File, io::Read };
		
		if self.is_dir() {
			Err(format!("Could not read dir \"{}\". Only able to read files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = File::open(self.path())?;
			let mut content:Vec<u8> = Vec::new();
			file.read_to_end(&mut content)?;
			Ok(content)
		}
	}
	
	/// Read a specific range of bytes from the file.
	pub fn read_range(&self, start:u64, end:u64) -> Result<Vec<u8>, Box<dyn Error>> {
		use std::{ fs::File, io::{ Read, Seek, SeekFrom } };

		if self.is_dir() {
			Err(format!("Could not read dir \"{}\". Only able to read files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = File::open(self.path())?;
			let mut buffer:Vec<u8> = vec![0; (end - start) as usize];
			file.seek(SeekFrom::Start(start))?;
			file.read_exact(&mut buffer)?;
			Ok(buffer)
		}
	}



	/* FILE WRITING METHODS */

	/// Create the file.
	pub fn create(&self) -> Result<(), Box<dyn Error>> {
		use std::fs::{ File, create_dir };

		let is_dir:bool = self.is_dir();
		if self.exists() {
			Err(format!("Could not create {} \"{}\". {} already exists.", if is_dir { "dir" } else { "file" }, self.path(), if is_dir { "Dir" } else { "File" }).into())
		} else {
			self.guarantee_parent_dir()?;
			if is_dir {
				create_dir(self.path()).map_err(|error| error.into())
			} else {
				File::create(&self.path())?;
				Ok(())
			}
		}
	}

	/// Guarantee that the file exists.
	pub fn guarantee_exists(&self) -> Result<(), Box<dyn Error>> {
		if !self.exists() {
			self.create()?;
		}
		Ok(())
	}

	/// Write a string to the file.
	pub fn write(&self, contents:&str) -> Result<(), Box<dyn Error>> {
		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else {
			self.write_bytes(contents.to_string().as_bytes())
		}
	}

	/// Write bytes to the file.
	pub fn write_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };
		
		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not write to file \"{}\". File does not exist.", self.path()).into())
		} else {
			self.guarantee_exists()?;
			let mut file:File = OpenOptions::new().write(true).truncate(true).open(self.path())?;
			file.write_all(data)?;
			Ok(())
		}
	}
	
	/// Read a specific range of bytes from the file.
	pub fn write_bytes_to_range(&self, start:u64, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::{ Write, Seek, SeekFrom } };

		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not write to file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = OpenOptions::new().write(true).open(self.path())?;
			file.seek(SeekFrom::Start(start))?;
			file.write_all(data).map_err(|error| error.into())
		}
	}

	/// Append bytes to the file.
	pub fn append_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };

		if self.is_dir() {
			Err(format!("Could not append to dir \"{}\". Only able to append to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not append to file \"{}\". File does not exist.", self.path()).into())
		} else {
			self.guarantee_exists()?;
			let mut file:File = OpenOptions::new().append(true).open(self.path())?;
			file.write_all(data)?;
			Ok(())
		}
	}



	/* FILE MOVING METHODS */

	/// Copy the file to another location. Returns the number of bytes written.
	pub fn copy_to(&self, target:&FileRef) -> Result<u64, Box<dyn Error>> {
		use std::fs::copy;

		if self.is_dir() {
			Err(format!("Could not copy dir \"{}\". Only able to copy files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not copy file \"{}\". File does not exist.", self.path()).into())
		} else {
			target.guarantee_parent_dir()?;
			copy(self.path(), target.path()).map_err(|error| error.into())
		}
	}



	/* FILE REMOVING METHODS */

	/// Delete the file.
	pub fn delete(&self) -> Result<(), Box<dyn Error>> {
		use std::fs::{ remove_dir_all, remove_file };

		if self.is_dir() {
			remove_dir_all(self.path()).map_err(|error| error.into())
		} else {
			remove_file(self.path()).map_err(|error| error.into())
		}
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



// Test with 1 thread!
#[cfg(test)]
mod tests {
	use super::*;
	
	

	/// Get a temp file.
	fn temp_file() -> FileRef {
		const TEMP_DIR:FileRef = FileRef::new_const("target/unit_testing_temp_files/");
		static mut FILE_INDEX:usize = 0;
		let file:FileRef = FileRef::new(&(TEMP_DIR.path().to_owned() + unsafe { &FILE_INDEX.to_string() } + ".txt"));
		if file.exists() {
			file.delete().expect("Could not delete existing temp file");
		}
		unsafe { FILE_INDEX += 1; }
		file
	}




	#[test]
	fn test_file_creation() {
		let temp_file:FileRef = temp_file();
		assert!(!temp_file.exists());
		temp_file.create().unwrap();
		assert!(temp_file.exists());
	}

	#[test]
	fn test_file_write_and_read() {
		let temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = "Hello, world!";
		temp_file.write(content).unwrap();

		let read_content = temp_file.read().unwrap();
		assert_eq!(content, read_content);
	}

	#[test]
	fn test_file_write_bytes_and_read_bytes() {
		let temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = b"Hello, binary world!";
		temp_file.write_bytes(content).unwrap();

		let read_content = temp_file.read_bytes().unwrap();
		assert_eq!(content, read_content.as_slice());
	}

	#[test]
	fn test_append_bytes() {
		let temp_file:FileRef = temp_file();
		
		temp_file.create().unwrap();

		let initial_content = "Hello";
		let append_content = ", world!";
		temp_file.write(initial_content).unwrap();
		temp_file.append_bytes(append_content.as_bytes()).unwrap();

		let read_content = temp_file.read().unwrap();
		assert_eq!(read_content, "Hello, world!");
	}

	#[test]
	fn test_read_range() {
		let temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = "Hello, world!";
		temp_file.write(content).unwrap();

		let range_content = temp_file.read_range(7, 12).unwrap();
		assert_eq!(std::str::from_utf8(&range_content).unwrap(), "world");
	}

	#[test]
	fn test_write_bytes_to_range() {
		let temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = "Hello, world!";
		temp_file.write(content).unwrap();

		let replacement = "Rust!";
		temp_file.write_bytes_to_range(7, replacement.as_bytes()).unwrap();

		let read_content = temp_file.read().unwrap();
		assert_eq!(read_content, "Hello, Rust!!");
	}

	#[test]
	fn test_file_deletion() {
		let temp_file:FileRef = temp_file();

		temp_file.create().unwrap();
		assert!(temp_file.exists());

		temp_file.delete().unwrap();
		assert!(!temp_file.exists());
	}

	#[test]
	fn test_file_copy() {
		let temp_file:FileRef = temp_file();
		let source_file_ref:FileRef = temp_file.clone();
		let target_file_ref:FileRef = FileRef::new(&(temp_file.path().to_owned() + "_target.txt"));

		source_file_ref.create().unwrap();
		let content = "Copy this content.";
		source_file_ref.write(content).unwrap();

		source_file_ref.copy_to(&target_file_ref).unwrap();
		assert!(target_file_ref.exists());

		let copied_content = target_file_ref.read().unwrap();
		assert_eq!(content, copied_content);

		target_file_ref.delete().unwrap();
	}
}
