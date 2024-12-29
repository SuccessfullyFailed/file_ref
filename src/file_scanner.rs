use crate::{ FileRef, SEPARATOR };
use std::error::Error;



pub type ResultFilter = Box<dyn Fn(&FileRef) -> bool>;
struct FileScannerCursor { dir:FileRef, last_parsed_file:Option<FileRef> }



pub struct FileScanner {
	source_dir:FileRef,
	include_files:bool,
	include_dirs:bool,
	results_filter:ResultFilter,
	recurse_filter:ResultFilter,

	cursor:FileScannerCursor
}
impl FileScanner {

	/* CONSTRUCTOR METHODS */

	/// Create a new filter.
	pub fn new(source_dir:&FileRef) -> FileScanner {
		FileScanner {
			source_dir: source_dir.clone().trim_end_matches(SEPARATOR),
			include_files: false,
			include_dirs: false,
			results_filter: Box::new(|_| true),
			recurse_filter: Box::new(|_| false),

			cursor: FileScannerCursor { dir: source_dir.clone(), last_parsed_file: None }
		}
	}

	/// Return self with a setting to include files in the scan results.
	pub fn include_files(mut self) -> Self {
		self.include_files = true;
		self
	}

	/// Return self with a setting to include directories in the scan results.
	pub fn include_dirs(mut self) -> Self {
		self.include_dirs = true;
		self
	}

	/// Return self with a result filter.
	pub fn with_filter(mut self, filter:ResultFilter) -> Self {
		self.results_filter = filter;
		self
	}

	/// Return self with a setting to recurse into sub-dirs.
	pub fn recurse(self) -> Self {
		self.with_recurse_filter(Box::new(|_| true))
	}

	/// Return self with a recurse filter.
	pub fn with_recurse_filter(mut self, filter:ResultFilter) -> Self {
		self.recurse_filter = filter;
		self
	}




	/* USAGE METHODS */

	/// Find the next matching file based on the cursor.
	fn find_next_file_at_cursor(&mut self) -> Result<FileRef, Box<dyn Error>> {
		
		// Try to find file at cursor.
		if let Ok(entry) = self.find_next_file_at(&self.cursor.dir, &self.cursor.last_parsed_file)  {
			if entry.is_dir() && (self.recurse_filter)(&entry) {
				self.cursor.dir = entry.clone();
				self.cursor.last_parsed_file = None;
			} else {
				self.cursor.dir = entry.parent_dir()?;
				self.cursor.last_parsed_file = Some(entry.clone());
			}
			return Ok(entry);
		}

		// Could not find any files.
		return Err("Could not find file.".into())
	}

	/// Find the next file from a specific directory.
	fn find_next_file_at(&self, dir:&FileRef, last_parsed_file:&Option<FileRef>) -> Result<FileRef, Box<dyn Error>> {
		use std::fs::read_dir;

		// Collect entries.
		let mut entries:Vec<FileRef> = Vec::new();
		if let Ok(results) = read_dir(dir.path()) {
			for path in results.flatten() {
				entries.push(FileRef::new(&path.path().display().to_string()));
			}
		}
		entries.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()));
		let start_entry_index:usize = last_parsed_file.as_ref().map(|skipping_entry| entries.iter().position(|entry| entry == skipping_entry).map(|index| index + 1).unwrap_or_default()).unwrap_or_default();
		entries = entries[start_entry_index..].to_vec();

		// Iterate through entries.
		for entry in &entries {
			let is_dir:bool = entry.is_dir();

			// Try to match result.
			if ((is_dir && self.include_dirs) || (!is_dir && self.include_files)) && (self.results_filter)(entry) {
				return Ok(entry.clone());
			}

			// Recurse into dirs.
			if is_dir && (self.recurse_filter)(entry) {
				if let Ok(file) = self.find_next_file_at(entry, &None) {
					return Ok(file);
				}
			}
		}

		// Could not find in any files in this dir, try to find one in parent dir if that is in temp_file dir.
		if let Ok(parent_dir) = dir.parent_dir() {
			if parent_dir.contains(self.source_dir.path()) {
				if let Ok(found) = self.find_next_file_at(&parent_dir, &Some(dir.clone())) {
					return Ok(found);
				}
			}
		}
		
		// Could not find any files.
		return Err("Could not find file.".into())

	}
}
impl Iterator for FileScanner {
	type Item = FileRef;

	fn next(&mut self) -> Option<Self::Item> {
		self.find_next_file_at_cursor().map(|result| Some(result)).unwrap_or_default()
	}
}



#[cfg(test)]
mod tests {
	use crate::unit_test_support::{ TempFile, UNIT_TEST_DIR };
	use super::*;

	fn create_test_structure() -> TempFile {
		let unit_test_dir:FileRef = UNIT_TEST_DIR.0 + "file_scanner/";
		let _ = [
			(unit_test_dir + "subdir1/subsubdir1").create(),
			(unit_test_dir + "subdir2").create(),
			(unit_test_dir + "file1.txt").create(),
			(unit_test_dir + "subdir1/file2.txt").create(),
			(unit_test_dir + "subdir1/subsubdir1/file3.txt").create(),
			(unit_test_dir + "subdir2/file4.txt").create()
		];
		TempFile(unit_test_dir.clone())
	}

	#[test]
	fn test_include_files() {
		let temp_file:TempFile = create_test_structure();

		let scanner:FileScanner = FileScanner::new(&temp_file.0).include_files();
		let results:Vec<_> = scanner.collect();

		// Check that only files are included
		assert!(results.iter().all(|f| !f.is_dir()));
		assert_eq!(results.len(), 1);
	}

	#[test]
	fn test_include_dirs() {
		let temp_file:TempFile = create_test_structure();
		let scanner:FileScanner = FileScanner::new(&temp_file.0).include_dirs();
		let results:Vec<_> = scanner.collect();

		// Check that only directories are included
		assert!(results.iter().all(|d| d.is_dir()));
		assert_eq!(results.len(), 2); // subdir1, subsubdir1, subdir2
	}

	#[test]
	fn test_include_files_and_dirs() {
		let temp_file:TempFile = create_test_structure();
		let scanner:FileScanner = FileScanner::new(&temp_file.0).include_files().include_dirs();
		let results:Vec<_> = scanner.collect();

		// Check that both files and directories are included
		assert!(results.len() > 0);
		assert!(results.iter().any(|e| e.is_dir()));
		assert!(results.iter().any(|e| !e.is_dir()));
	}

	#[test]
	fn test_filter() {
		let temp_file:TempFile = create_test_structure();
		let scanner:FileScanner = FileScanner::new(&temp_file.0).include_files().recurse().with_filter(Box::new(|f| f.name().ends_with(".txt")));

		let results:Vec<_> = scanner.collect();

		// Check that only .txt files are included
		assert!(results.iter().all(|f| f.name().ends_with(".txt")));
		assert_eq!(results.len(), 4);
	}

	#[test]
	fn test_recursion() {
		let temp_file:TempFile = create_test_structure();
		let scanner:FileScanner = FileScanner::new(&temp_file.0).include_files().recurse();
		let results:Vec<_> = scanner.collect();

		// Check that all files are included across all directories
		assert_eq!(results.len(), 4); // file1, file2, file3, file4
	}

	#[test]
	fn test_recurse_filter() {
		let temp_file:TempFile = create_test_structure();
		let scanner:FileScanner = FileScanner::new(&temp_file.0).include_files().with_recurse_filter(Box::new(|d| d.name() != "subdir1"));

		let results:Vec<_> = scanner.collect();

		// Check that files under "subdir1" are not included
		assert!(results.iter().all(|f| !f.path().contains("subdir1")));
	}

	#[test]
	fn test_root_is_file() {
		let temp_file:TempFile = create_test_structure();
		let scanner:FileScanner = FileScanner::new(&FileRef::new(&(temp_file.0.path().to_owned() + "file1.txt"))).include_files().include_dirs().recurse();

		let results:Vec<_> = scanner.collect();

		// Check that only .txt files are included
		assert!(results.iter().all(|f| f.name().ends_with("1.txt")));
		assert_eq!(results.len(), 0);
	}
}
