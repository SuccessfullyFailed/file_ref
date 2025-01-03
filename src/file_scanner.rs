use crate::{ FileRef, SEPARATOR };



pub type ResultFilter = Box<dyn Fn(&FileRef) -> bool>;
struct FileScannerCursor {
	parsed_self:bool,
	current_dir:Option<FileRef>,
	entries_in_current_dir:Option<Vec<FileRef>>,
	previous_entry_index:Option<usize>,
	entries_cache:Vec<(FileRef, Vec<FileRef>)>
}



pub struct FileScanner {
	root_dir:FileRef,
	include_self:bool,
	include_files:bool,
	include_dirs:bool,
	results_filter:ResultFilter,
	recurse_filter:ResultFilter,

	cursor:FileScannerCursor
}
impl FileScanner {

	/* CONSTRUCTOR METHODS */

	/// Create a new filter.
	pub fn new(root_dir:&FileRef) -> FileScanner {
		let root_dir:FileRef = root_dir.clone().absolute().trim_end_matches(SEPARATOR);
		FileScanner {
			root_dir: root_dir.clone(),
			include_self: false,
			include_files: false,
			include_dirs: false,
			results_filter: Box::new(|_| true),
			recurse_filter: Box::new(|_| false),

			cursor: FileScannerCursor {
				parsed_self: false,
				current_dir: None,
				entries_in_current_dir: None,
				previous_entry_index: None,
				entries_cache: Vec::new()
			}
		}
	}

	/// Include source dir in final results.
	pub fn include_self(mut self) -> Self {
		self.include_self = true;
		self
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

	/// Return self with a result filter. Overwrites the default filter function to filter out entries during the search process, rather than after being returned.
	pub fn filter<T>(mut self, filter:T) -> Self where T:Fn(&FileRef) -> bool + 'static {
		self.results_filter = Box::new(filter);
		self
	}

	/// Return self with a setting to recurse into sub-dirs.
	pub fn recurse(self) -> Self {
		self.recurse_filter(|_| true)
	}

	/// Return self with a recurse filter.
	pub fn recurse_filter<T>(mut self, filter:T) -> Self where T:Fn(&FileRef) -> bool + 'static {
		self.recurse_filter = Box::new(filter);
		self
	}




	/* USAGE METHODS */

	/// Find the next matching file based on the cursor.
	fn find_next_at_cursor(&mut self) -> Option<FileRef> {
		
		// Parse self if necessary.
		if !self.cursor.parsed_self {
			self.cursor.parsed_self = true;
			if self.include_self && self.root_dir.exists() && (self.results_filter)(&self.root_dir) {
				return Some(self.root_dir.clone());
			}
		}

		// If no current dir exists, start at root dir.
		if self.cursor.current_dir.is_none() {
			self.cursor.current_dir = Some(self.root_dir.clone());
		}

		// Try to find the next item in the current dir.
		if let Some((entry_index, entry)) = self.find_next_in_current_dir() {
			self.cursor.previous_entry_index = Some(entry_index);
			return Some(entry);
		}

		// If not found in current dir, try to recurse into sub-dir.
		if let Some(current_dir) = self.cursor.current_dir.clone() {
			if let Some(sub_dir) = self.entries_in_dir(&current_dir).to_owned().iter().filter(|entry| entry.is_dir() && (self.recurse_filter)(*entry)).next() {
				if let Some(entry) = self.find_in_new_dir(sub_dir.clone()) {
					return Some(entry);
				}
			}
		}

		// If not found in any sub-dirs, keep moving to parent dirs while keeping inside the source dir.
		if let Some(mut current_dir) = self.cursor.current_dir.clone() {
			if let Ok(mut parent_dir) = current_dir.parent_dir() {
				while parent_dir.contains(&self.root_dir.path()) {

					// Find new sub-dirs in the parent dir, below the current dir.
					let parent_dir_sub_dirs:Vec<FileRef> = parent_dir.list_dirs();
					if let Some(own_index_in_parent_dir) = parent_dir_sub_dirs.iter().position(|dir| dir == &current_dir) {
						if let Some(sub_dir) = parent_dir_sub_dirs[own_index_in_parent_dir + 1..].iter().filter(|dir| (self.recurse_filter)(&dir)).next() {
							if let Some(entry) = self.find_in_new_dir(sub_dir.clone()) {
								return Some(entry);
							}
						}
					} else {
						eprintln!("FileRef scanner Could not recurse back to parent of '{current_dir}'. Dir does not seem to exist anymore.");
					}

					// Move to parent dir.
					if let Ok(parent_parent_dir) = parent_dir.parent_dir() {
						current_dir = parent_dir.clone();
						parent_dir = parent_parent_dir;
					}
				}
			}
		}
		
		// No results.
		None
	}

	/// Move the cursor to a new place and find the next file there.
	fn find_in_new_dir(&mut self, new_dir:FileRef) -> Option<FileRef> {
		self.cursor.current_dir = Some(new_dir);
		self.cursor.previous_entry_index = None;
		self.cursor.entries_in_current_dir = None;
		self.find_next_at_cursor()
	}

	/// Try to find the next entry in the current dir.
	fn find_next_in_current_dir(&mut self) -> Option<(usize, FileRef)> {

		// List and store entries in current dir.
		if self.cursor.entries_in_current_dir.is_none() {
			if let Some(current_dir) = self.cursor.current_dir.clone() {
				self.cursor.entries_in_current_dir = Some(self.entries_in_dir(&current_dir).to_owned());
			}
		}

		// Find suitable entries in current dir.
		if let Some(current_dir_entries) = &self.cursor.entries_in_current_dir {
			let start_index:usize = self.cursor.previous_entry_index.map(|index| index + 1).unwrap_or(0);
			for (entry_index, entry) in current_dir_entries.iter().enumerate().skip(start_index) {
				if self.entry_matches_filter(entry) {
					return Some((entry_index, entry.clone()));
				}
			}
		}

		// None found.
		None
	}

	/// Check if a file or dir matches the filters.
	fn entry_matches_filter(&self, entry:&FileRef) -> bool {
		(if entry.is_dir() { self.include_dirs } else { self.include_files }) && (self.results_filter)(entry)
	}

	/// List the entries in a specific dir.
	fn entries_in_dir(&mut self, dir:&FileRef) -> &Vec<FileRef> {
		use std::fs::read_dir;

		// Find in cache.
		if let Some(cache_index) = self.cursor.entries_cache.iter().position(|(cache_dir, _)| cache_dir == dir) {
			return &self.cursor.entries_cache[cache_index].1;
		}

		// List entries in actual folder and store in cache.
		let entries:Vec<FileRef> = read_dir(dir.path()).map(|results| results.flatten().map(|dir_entry| FileRef::new(dir_entry.path().to_str().unwrap())).collect::<Vec<FileRef>>()).unwrap_or_default();
		self.cursor.entries_cache.push((dir.clone(), entries));
		&self.cursor.entries_cache.last().unwrap().1
	}
}
impl Iterator for FileScanner {
	type Item = FileRef;

	fn next(&mut self) -> Option<Self::Item> {
		self.find_next_at_cursor()
	}
}