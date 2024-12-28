use std::{error::Error, ops::{ Deref, DerefMut }};
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

	/// Check if the files exists.
	pub fn exists(&self) -> bool {
		std::path::Path::new(&self.path()).exists() && std::fs::metadata(&self.path()).map(|data| data.is_file()).unwrap_or(false)
	}
	
	/// Check if the file can be accessed.
	pub fn is_accessible(&self) -> bool {
		std::fs::File::open(&self.path()).is_ok()
	}



	/* FILE READING METHODS */

	/// Read the contents of the file as a string.
	pub fn read(&self) -> Result<String, Box<dyn Error>> {
		use std::{ fs::File, io::Read };
		
		if self.exists() {
			let mut file:File = File::open(self.path())?;
			let mut contents:String = String::new();
			file.read_to_string(&mut contents)?;
			Ok(contents)
		} else {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		}
	}

	/// Read the contents of the file as bytes.
	pub fn read_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		use std::{ fs::File, io::Read };
		
		if self.exists() {
			let mut file:File = File::open(self.path())?;
			let mut content:Vec<u8> = Vec::new();
			file.read_to_end(&mut content)?;
			Ok(content)
		} else {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		}
	}
	
	/// Read a specific range of bytes from the file.
	pub fn read_range(&self, start:u64, end:u64) -> Result<Vec<u8>, Box<dyn Error>> {
		use std::{ fs::File, io::{ Read, Seek, SeekFrom } };

		if self.exists() {
			let mut file:File = File::open(self.path())?;
			let mut buffer:Vec<u8> = vec![0; (end - start) as usize];
			file.seek(SeekFrom::Start(start))?;
			file.read_exact(&mut buffer)?;
			Ok(buffer)
		} else {
			Err(format!("Could not read range in file \"{}\". File does not exist.", self.path()).into())
		}
	}



	/* FILE WRITING METHODS */

	/// Create the file.
	pub fn create(&self) -> Result<(), Box<dyn Error>> {
		use std::fs::File;

		if self.exists() {
			Err(format!("Could not create file \"{}\". File already exists.", self.path()).into())
		} else {
			File::create(&self.path())?;
			Ok(())
		}
	}

	/// Guarantee that the file exists.
	pub fn guarantee_exists(&self) -> Result<(), Box<dyn Error>> {
		self.guarantee_parent_dir()?;
		if !self.exists() {
			self.create()?;
		}
		Ok(())
	}

	/// Write a string to the file.
	pub fn write(&self, contents:&str) -> Result<(), Box<dyn Error>> {
		self.write_bytes(contents.to_string().as_bytes())
	}

	/// Write bytes to the file.
	pub fn write_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };
		
		self.guarantee_exists()?;
		let mut file:File = OpenOptions::new().write(true).truncate(true).open(self.path())?;
		file.write_all(data)?;
		Ok(())
	}
	
	/// Read a specific range of bytes from the file.
	pub fn write_bytes_to_range(&self, start:u64, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::{ Write, Seek, SeekFrom } };

		if self.exists() {
			let mut file:File = OpenOptions::new().write(true).open(self.path())?;
			file.seek(SeekFrom::Start(start))?;
			file.write_all(data)?;
			Ok(())
		} else {
			Err(format!("Could not write to file range \"{}\". File does not exist.", self.path()).into())
		}
	}

	/// Append bytes to the file.
	pub fn append_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };

		// Make sure file exists.
		if !self.exists() {
			if let Err(error) = self.create() {
				return Err(format!("Could not append to file \"{}\". {}", self.path(), error).into());
			}
		}

		// Write to file.
		let mut file:File = OpenOptions::new().append(true).open(self.path())?;
		file.write_all(data)?;
		Ok(())
	}



	/* FILE MOVING METHODS */

	/// Copy the file to another location. Returns the number of bytes written.
	pub fn copy_to(&self, target:&FileRef) -> Result<u64, Box<dyn Error>> {
		target.guarantee_parent_dir()?;
		std::fs::copy(self.path(), target.path()).map_err(|error| error.into())
	}



	/* FILE REMOVING METHODS */

	/// Delete the file.
	pub fn delete(&self) -> Result<(), Box<dyn Error>> {
		std::fs::remove_file(self.path()).map_err(|error| error.into())
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
	use crate::DirRef;
	use super::*;
	
	

	/// Get a temp file.
	fn temp_file() -> FileRef {
		const TEMP_FILE:FileRef = FileRef::new_const("target/unit_testing_temp_files/test_file.txt");
		if TEMP_FILE.exists() {
			TEMP_FILE.delete().expect("Could not delete temp file");
		}
		TEMP_FILE.clone()
	}




	#[test]
	fn test_file_creation() {
		let mut temp_file:FileRef = temp_file();
		assert!(!temp_file.exists());
		temp_file.create().unwrap();
		assert!(temp_file.exists());
	}

	#[test]
	fn test_file_write_and_read() {
		let mut temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = "Hello, world!";
		temp_file.write(content).unwrap();

		let read_content = temp_file.read().unwrap();
		assert_eq!(content, read_content);
	}

	#[test]
	fn test_file_write_bytes_and_read_bytes() {
		let mut temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = b"Hello, binary world!";
		temp_file.write_bytes(content).unwrap();

		let read_content = temp_file.read_bytes().unwrap();
		assert_eq!(content, read_content.as_slice());
	}

	#[test]
	fn test_append_bytes() {
		let mut temp_file:FileRef = temp_file();
		
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
		let mut temp_file:FileRef = temp_file();

		temp_file.create().unwrap();

		let content = "Hello, world!";
		temp_file.write(content).unwrap();

		let range_content = temp_file.read_range(7, 12).unwrap();
		assert_eq!(std::str::from_utf8(&range_content).unwrap(), "world");
	}

	#[test]
	fn test_write_bytes_to_range() {
		let mut temp_file:FileRef = temp_file();

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
		let mut temp_file:FileRef = temp_file();

		temp_file.create().unwrap();
		assert!(temp_file.exists());

		temp_file.delete().unwrap();
		assert!(!temp_file.exists());
	}

	#[test]
	fn test_file_copy() {
		let mut temp_file:FileRef = temp_file();
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
