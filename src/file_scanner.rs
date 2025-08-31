use crate::{ FileRef, SEPARATOR };



type ResultFilter = Box<dyn Fn(&FileRef) -> bool>;
struct ScanSettings {
	include_self:bool,
	include_files:bool,
	include_dirs:bool,
	results_filter:ResultFilter,
	recurse_filter:ResultFilter
}



pub struct FileScanner {
	scan_settings:ScanSettings,
	sub_dir_scanner:SubDirScanner
}
impl FileScanner {

	/* CONSTRUCTOR METHODS */

	/// Create a new filter.
	pub fn new(root_dir:&FileRef) -> FileScanner {
		let root_dir:FileRef = root_dir.clone().absolute().trim_end_matches(SEPARATOR);
		FileScanner {
			scan_settings: ScanSettings {
				include_self: false,
				include_files: false,
				include_dirs: false,
				results_filter: Box::new(|_| true),
				recurse_filter: Box::new(|_| false),
			},
			sub_dir_scanner: SubDirScanner::new(root_dir)
		}
	}

	/// Include source dir in final results.
	pub fn include_self(mut self) -> Self {
		self.scan_settings.include_self = true;
		self
	}

	/// Return self with a setting to include files in the scan results.
	pub fn include_files(mut self) -> Self {
		self.scan_settings.include_files = true;
		self
	}

	/// Return self with a setting to include directories in the scan results.
	pub fn include_dirs(mut self) -> Self {
		self.scan_settings.include_dirs = true;
		self
	}

	/// Return self with a result filter. Overwrites the default filter function to filter out entries during the search process, rather than after being returned.
	pub fn filter<T>(mut self, filter:T) -> Self where T:Fn(&FileRef) -> bool + 'static {
		self.scan_settings.results_filter = Box::new(filter);
		self
	}

	/// Return self with a setting to recurse into sub-dirs.
	pub fn recurse(self) -> Self {
		self.recurse_filter(|_| true)
	}

	/// Return self with a recurse filter.
	pub fn recurse_filter<T>(mut self, filter:T) -> Self where T:Fn(&FileRef) -> bool + 'static {
		self.scan_settings.recurse_filter = Box::new(filter);
		self
	}
}
impl Iterator for FileScanner {
	type Item = FileRef;

	fn next(&mut self) -> Option<Self::Item> {
		self.sub_dir_scanner.get(&self.scan_settings)
	}
}



struct SubDirScanner {
	dir:FileRef,
	parsed_self:bool,
	files_in_dir:Option<Vec<FileRef>>,
	dirs_in_dir:Option<Vec<FileRef>>,
	sub_scanners:Option<Vec<SubDirScanner>>
}
impl SubDirScanner {

	/// Create a new recursive sub-dir scanner.
	fn new(dir:FileRef) -> SubDirScanner {
		SubDirScanner {
			dir,
			parsed_self: false,
			files_in_dir: None,
			dirs_in_dir: None,
			sub_scanners: None
		}
	}

	/// Get the next file.
	fn get(&mut self, scan_settings:&ScanSettings) -> Option<FileRef> {

		// Try Self.
		if scan_settings.include_self && !self.parsed_self {
			self.parsed_self = true;
			if (scan_settings.results_filter)(&self.dir) {
				return Some(self.dir.clone());
			}
		}

		// Scan entries in this dir.
		if self.files_in_dir.is_none() || self.sub_scanners.is_none() || self.sub_scanners.is_none() {
			let dir_entries:Vec<FileRef> = Self::get_dir_raw_entries(&self.dir);
			let mut files:Vec<FileRef> = Vec::new();
			let mut dirs:Vec<FileRef> = Vec::new();
			for entry in dir_entries {
				if entry.extension().is_some() {
					files.push(entry);
				} else {
					dirs.push(entry);
				}
			}
			self.sub_scanners = Some(dirs.iter().filter(|dir| (scan_settings.recurse_filter)(dir)).map(|dir| SubDirScanner::new(dir.clone())).collect::<Vec<SubDirScanner>>());
			self.files_in_dir = Some(files);
			self.dirs_in_dir = Some(dirs);
		}

		// Try files in dir.
		if scan_settings.include_files {
			if let Some(files) = &mut self.files_in_dir {
				while !files.is_empty() {
					let file:FileRef = files.remove(0);
					if (scan_settings.results_filter)(&file) {
						return Some(file);
					}
				}
			}
		}

		// Try dirs in dir.
		if scan_settings.include_dirs {
			if let Some(dirs) = &mut self.dirs_in_dir {
				while !dirs.is_empty() {
					let dir:FileRef = dirs.remove(0);
					if (scan_settings.results_filter)(&dir) {
						return Some(dir);
					}
				}
			}
		}

		// Try sub-scanners.
		if let Some(sub_scanners) = &mut self.sub_scanners {
			while !sub_scanners.is_empty() {
				let sub_scanner:&mut SubDirScanner = &mut sub_scanners[0];
				if let Some(result) = sub_scanner.get(&scan_settings) {
					return Some(result);
				}
				sub_scanners.remove(0);
			}
		}

		// None found.
		None
	}

	/// Get all files and folders in the given directory non-recursive.
	fn get_dir_raw_entries(dir:&FileRef) -> Vec<FileRef> {
		std::fs::read_dir(dir.path())
			.map(|results|
				results
					.flatten()
					.map(|dir_entry|
						FileRef::new(dir_entry.path().to_str().unwrap())
					)
					.collect::<Vec<FileRef>>()
			).unwrap_or_default()
	}
}