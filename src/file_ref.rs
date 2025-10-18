use std::{ error::Error, time::SystemTime, fs::{ Metadata, Permissions }, ops::{ Add, AddAssign } };
use core::fmt::{ self, Display, Debug, Formatter };
use crate::FileScanner;



// Most could be chars, but will be used as str's mainly, so this stops the program from converting.
pub(crate) const SEPARATOR:&str = "/";
const INVALID_SEPARATOR:&str = "\\";
const DOUBLE_SEPARATOR:&str = "//";
const DISK_SEPARATOR:&str = ":";



#[derive(Clone, Eq, PartialOrd, Ord)]
pub(crate) enum FilePath {
	StaticStr(&'static str),
	Owned(String)
}
impl FilePath {

	/// Create a new owned path.
	pub fn new(path:&str) -> FilePath {
		
		// Fix incorrect or messy separators.
		let mut path:String = path.replace(INVALID_SEPARATOR, SEPARATOR);
		while path.contains(DOUBLE_SEPARATOR) {
			path = path.replace(DOUBLE_SEPARATOR, SEPARATOR);
		}


		// Remove '..' where possible.
		let mut nodes:Vec<&str> = path.split(SEPARATOR).collect();
		if nodes.len() >= 2 {
			let mut index:usize = 1;
			while index < nodes.len() {
				if nodes[index] == ".." && nodes[index - 1] != ".." {
					nodes.remove(index);
					nodes.remove(index - 1);
					index = 0; // Restart after all modifications, required tow fix paths like a/b/../..
				} else {
					index += 1;
				}
			}
		}
		
		// Remove './' if it's not the full path.
		if nodes.contains(&".") && !nodes.iter().all(|node| *node == ".") {
			nodes.retain(|node| *node != ".");
		}

		// Return new file.
		FilePath::Owned(nodes.join(SEPARATOR))
	}

	/// Create a new statically borrowed path. This may behave unexpectedly for messy paths (using '.' or '..').
	pub const fn new_const(path:&'static str) -> FilePath {
		FilePath::StaticStr(path)
	}

	/// Get the raw path.
	pub fn path(&self) -> &str {
		match self {
			FilePath::StaticStr(path) => *path,
			FilePath::Owned(path) => path.as_str()
		}
	}
}
impl PartialEq<FilePath> for FilePath {
	fn eq(&self, other:&FilePath) -> bool {
		self.path() == other.path()
	}
}



#[derive(Clone, Eq, PartialOrd, Ord)]
pub struct FileRef(FilePath);
impl FileRef {

	/* CONSTRUCTOR METHODS */

	/// Create a new owned path.
	pub fn new(path:&str) -> FileRef {
		FileRef(FilePath::new(path))
	}

	/// Create a new statically borrowed path. This may behave unexpectedly for messy paths (using '.' or '..').
	pub const fn new_const(path:&'static str) -> FileRef {
		FileRef(FilePath::new_const(path))
	}

	/// Get the working dir of the application.
	pub fn working_dir() -> FileRef {
		FileRef::new(&std::env::current_dir().unwrap().display().to_string())
	}

	/// Return self with a absolute path.
	pub fn absolute(self) -> FileRef {
		if self.is_absolute_path() {
			self
		} else {
			FileRef::working_dir() + "/" + self.path()
		}
	}

	/// Return self with a relatvie path.
	pub fn relative(self) -> FileRef {
		let working_dir:FileRef = FileRef::working_dir();
		if self.is_relative_path() || !self.contains(working_dir.path()) {
			self
		} else {
			self.replace((working_dir + "/").path(), "")
		}
	}

	/// Create a relative path from self to another path.
	pub fn relative_path_to(&self, target:&FileRef) -> FileRef {

		// Process both paths as equal as possible.
		let source_path:FileRef = self.clone().absolute();
		let target_path:FileRef = target.clone().absolute();
		let mut source_steps:Vec<&str> = source_path.path_nodes();
		let mut target_steps:Vec<&str> = target_path.path_nodes();

		// Remove equal parts.
		while !source_steps.is_empty() && !target_steps.is_empty() && source_steps[0] == target_steps[0] {
			source_steps.remove(0);
			target_steps.remove(0);
		}

		// Calculate and return path from remaining steps.
		let relative_path:String = [source_steps.iter().map(|_| "..").collect::<Vec<&str>>(), target_steps].iter().flatten().copied().collect::<Vec<&str>>().join(SEPARATOR);
		FileRef::new(&relative_path)
	}



	/* PROPERTY GETTER METHODS */

	/// Get the raw path.
	pub fn path(&self) -> &str {
		self.0.path()
	}

	/// Get the directory the file is in.
	pub fn parent_dir(&self) -> Result<FileRef, Box<dyn Error>> {
		let path:&str = self.path();
		let nodes:Vec<&str> = self.path_nodes();
		if *nodes.last().unwrap_or(&"") == ".." {
			Ok(self.clone() + "/..")
		} else if nodes.len() <= 1 {
			if self.is_relative_path() {
				self.clone().absolute().parent_dir()
			} else {
				Err(format!("Could not get dir of file \"{path}\", as it only contains the file name.").into())
			}
		} else {
			let parent_dir_len:usize = nodes[..nodes.len() - 1].join(SEPARATOR).len();
			Ok(FileRef::new(&path[..parent_dir_len]))
		}
	}

	/// Get a list of nodes in the path.
	pub(crate) fn path_nodes(&self) -> Vec<&str> {
		let mut parts:Vec<&str> = self.path().split(SEPARATOR).collect();
		while parts.last() == Some(&"") {
			parts.remove(parts.len() - 1);
		}
		parts
	}

	/// Get the last node of the path.
	pub(crate) fn last_node(&self) -> &str {
		self.path().split(SEPARATOR).last().unwrap_or_default()
	}

	/// Check if the path is a relative or absolute path.
	pub fn is_absolute_path(&self) -> bool {
		self.contains(DISK_SEPARATOR)
	}

	/// Check if the path is a relative or absolute path.
	pub fn is_relative_path(&self) -> bool {
		!self.is_absolute_path()
	}



	/* PROPERTY GETTER METHODS */

	/// Get the name of the file/dir.
	pub fn name(&self) -> &str {
		self.last_node()
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

	/// Check if self is a dir.
	pub fn is_dir(&self) -> bool {
		// Check metadata if exists, otherwise check extension.
		if self.exists() {
			if let Ok(metadata) = std::fs::metadata(&self.path()) {
				return metadata.is_dir();
			}
		}
		self.extension().map(|extension| extension.is_empty()).unwrap_or(true)
	}

	/// Check if self is a file.
	pub fn is_file(&self) -> bool {
		!self.is_dir()
	}
	
	/// Check if the file can be accessed.
	pub fn is_accessible(&self) -> bool {
		if self.is_dir() { true } else { std::fs::File::open(&self.path()).is_ok() }
	}



	/* METADATA METHODS */
	
	/// Get the metadata of the file.
	fn metadata(&self) -> Result<Metadata, Box<dyn Error>> {
		if self.is_dir() {
			Err(format!("Could not get metadata, file {self}, path is a directory.").into())
		} else if !self.exists() {
			Err(format!("Could not get metadata, file {self} does not exist").into())
		} else {
			Ok(std::fs::File::open(self.path())?.metadata()?)
		}
	}

	/// Get the amount of bytes the file is.
	pub fn bytes_size(&self) -> u64 {
		if !self.exists() {
			0
		} else if self.is_dir() {
			self.list_files_recurse().iter().map(|file| file.bytes_size()).sum()
		} else {
			self.metadata().map(|data| data.len()).unwrap_or(0)
		}
	}

	/// Get the creation time of the file.
	pub fn get_time_creation(&self) -> Result<SystemTime, Box<dyn Error>> {
		match self.metadata()?.created() {
			Ok(time) => Ok(time),
			Err(error) => Err(error.into())
		}
	}

	/// Get the modification time of the file.
	pub fn get_time_modification(&self) -> Result<SystemTime, Box<dyn Error>> {
		match self.metadata()?.modified() {
			Ok(time) => Ok(time),
			Err(error) => Err(error.into())
		}
	}

	/// Get the last accessed time of the file.
	pub fn get_time_accessed(&self) -> Result<SystemTime, Box<dyn Error>> {
		match self.metadata()?.accessed() {
			Ok(time) => Ok(time),
			Err(error) => Err(error.into())
		}
	}

	/// Get the file's permissions.
	pub fn permissions(&self) -> Result<Permissions, Box<dyn Error>> {
		Ok(self.metadata()?.permissions())
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

	/// If the file/dir does not exist, create it.
	pub fn guarantee_exists(&self) -> Result<(), Box<dyn Error>> {
		if !self.exists() {
			self.create()?;
		}
		Ok(())
	}

	/// If the parent dir does not exist, create it.
	pub fn guarantee_parent_dir(&self) -> Result<(), Box<dyn Error>> {
		let parent_dir:FileRef = self.parent_dir()?;
		if !parent_dir.exists() {
			parent_dir.guarantee_parent_dir()?;
			parent_dir.create()?;
		}
		Ok(())
	}

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

	/// Write a string to the file.
	pub fn write(&self, contents:String) -> Result<(), Box<dyn Error>> {
		self._write(contents, false)
	}

	/// Write a string to the file and wait until the file has finished.
	pub fn write_await(&self, contents:String) -> Result<(), Box<dyn Error>> {
		self._write(contents, true)
	}

	/// Write a string to the file.
	fn _write(&self, contents:String, await_finish:bool) -> Result<(), Box<dyn Error>> {
		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else {
			self._write_bytes(contents.as_bytes(), await_finish)
		}
	}

	/// Write bytes to the file.
	pub fn write_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		self._write_bytes(data, false)
	}

	/// Write bytes to the file and wait until the file has finished.
	pub fn write_bytes_await(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		self._write_bytes(data, true)
	}

	/// Write bytes to the file.
	fn _write_bytes(&self, data:&[u8], await_finish:bool) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };
		
		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else {
			self.guarantee_exists()?;
			let mut file:File = OpenOptions::new().write(true).truncate(true).open(self.path())?;
			file.write_all(data)?;
			if await_finish {
				file.flush()?;
			}
			Ok(())
		}
	}

	/// Read a specific range of bytes from the file.
	pub fn write_bytes_to_range(&self, start:u64, data:&[u8]) -> Result<(), Box<dyn Error>> {
		self._write_bytes_to_range(start, data, false)
	}

	/// Read a specific range of bytes from the file and wait until the file has finished.
	pub fn write_bytes_to_range_await(&self, start:u64, data:&[u8]) -> Result<(), Box<dyn Error>> {
		self._write_bytes_to_range(start, data, true)
	}

	/// Read a specific range of bytes from the file.
	fn _write_bytes_to_range(&self, start:u64, data:&[u8], await_finish:bool) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::{ Write, Seek, SeekFrom } };

		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not write to file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = OpenOptions::new().write(true).open(self.path())?;
			file.seek(SeekFrom::Start(start))?;
			file.write_all(data)?;
			if await_finish {
				file.flush()?;
			}
			Ok(())
		}
	}

	/// Append a string to the file. Writes it to the file on disk.
	pub fn append(&self, contents:String) -> Result<(), Box<dyn Error>> {
		self._append_bytes(contents.as_bytes(), false)
	}

	/// Append a string to the file and wait until the file has finished. Writes it to the file on disk.
	pub fn append_await(&self, contents:String) -> Result<(), Box<dyn Error>> {
		self._append_bytes(contents.as_bytes(), true)
	}

	/// Append bytes to the file.
	pub fn append_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		self._append_bytes(data, false)
	}

	/// Append bytes to the file and wait until the file has finished.
	pub fn append_bytes_await(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		self._append_bytes(data, true)
	}

	/// Append bytes to the file.
	fn _append_bytes(&self, data:&[u8], await_finish:bool) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };

		if self.is_dir() {
			Err(format!("Could not append to dir \"{}\". Only able to append to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not append to file \"{}\". File does not exist.", self.path()).into())
		} else {
			self.guarantee_exists()?;
			let mut file:File = OpenOptions::new().append(true).open(self.path())?;
			file.write_all(data)?;
			if await_finish {
				file.flush()?;
			}
			Ok(())
		}
	}



	/* FILE MOVING METHODS */

	/// Move the file to another location.
	pub fn move_to(&self, target:&FileRef) -> Result<(), Box<dyn Error>> {
		use std::fs::rename;

		if self.is_dir() {
			Err(format!("Could not copy dir \"{}\". Only able to copy files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not copy file \"{}\". File does not exist.", self.path()).into())
		} else {
			target.guarantee_parent_dir()?;
			rename(self.path(), target.path()).map_err(|error| error.into())
		}
	}

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



	/* QUICK SCANNER METHODS */

	/// Create a basic scanner on this dir.
	pub fn scanner(&self) -> FileScanner {
		FileScanner::new(self)
	}

	/// Create a file-scanner on this dir that lists all files.
	pub fn list_files(&self) -> Vec<FileRef> {
		self.scanner().include_files().collect()
	}
	
	/// Create a file-scanner on this dir that lists all files recursively.
	pub fn list_files_recurse(&self) -> Vec<FileRef> {
		self.scanner().include_files().recurse().collect()
	}

	/// Create a file-scanner on this dir that lists all dirs.
	pub fn list_dirs(&self) -> Vec<FileRef> {
		self.scanner().include_dirs().collect()
	}

	/// Create a file-scanner on this dir that lists all dirs.
	pub fn list_dirs_recurse(&self) -> Vec<FileRef> {
		self.scanner().include_dirs().recurse().collect()
	}
}
impl PartialEq<FileRef> for FileRef {
	fn eq(&self, other:&FileRef) -> bool {
		self.path() == other.path() || self.clone().absolute().path() == other.clone().absolute().path()
	}
}
impl Add<&str> for FileRef {
	type Output = FileRef;

	fn add(self, rhs:&str) -> Self::Output {
		FileRef::new(&(self.path().to_owned() + rhs))
	}
}
impl AddAssign<&str> for FileRef {
	fn add_assign(&mut self, rhs:&str) {
		*self = FileRef::new(&(self.path().to_owned() + rhs));
	}
}
impl Display for FileRef {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.path())
	}
}
impl Debug for FileRef {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.path())
	}
}



/* STR INHERITED METHODS */
macro_rules! impl_inherit_str {

	// Case for methods without arguments.
	($fn_name:ident, $output_type:ty) => {
		impl FileRef {
			pub fn $fn_name(&self) -> $output_type {
				self.path().$fn_name()
			}
		}
	};

	// Case for methods with arguments.
	($fn_name:ident, $output_type:ty, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FileRef {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> $output_type {
				self.path().$fn_name($($arg_name),*)
			}
		}
	};

	// Case for methods returning `FileRef`.
	(ret_self $fn_name:ident) => {
		impl FileRef {
			pub fn $fn_name(&self) -> FileRef {
				FileRef::new(&self.path().$fn_name())
			}
		}
	};

	// Case for methods returning `FileRef` with arguments.
	(ret_self $fn_name:ident, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FileRef {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> FileRef {
				FileRef::new(&self.path().$fn_name($($arg_name),*))
			}
		}
	};

	// Case for methods returning `Option<FileRef>`.
	(ret_self_opt $fn_name:ident) => {
		impl FileRef {
			pub fn $fn_name(&self) -> Option<FileRef> {
				self.path().$fn_name().map(|path| FileRef::new(path))
			}
		}
	};

	// Case for methods returning `Option<FileRef>` with arguments.
	(ret_self_opt $fn_name:ident, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FileRef {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> Option<FileRef> {
				self.path().$fn_name($($arg_name),*).map(|path| FileRef::new(path))
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
impl_inherit_str!(ret_self trim_start_matches, (pat:&str));
impl_inherit_str!(ret_self trim_end);
impl_inherit_str!(ret_self trim_end_matches, (pat:&str));
impl_inherit_str!(ret_self repeat, (n:usize));
impl_inherit_str!(ret_self replace, (from:&str, to:&str));
impl_inherit_str!(ret_self_opt strip_prefix, (prefix:&str));
impl_inherit_str!(ret_self_opt strip_suffix, (suffix:&str));