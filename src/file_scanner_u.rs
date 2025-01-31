#[cfg(test)]
mod tests {
	use crate::{ FileRef, FileScanner, unit_test_support::TempFile };



	fn create_test_structure() -> TempFile {
		let unit_test_dir:TempFile = TempFile::new(None);
		let _ = [
			FileRef::new(unit_test_dir.path()).create(),
			FileRef::new(&(unit_test_dir.path().to_owned() + "/subdir1/sub_subdir1")).create(),
			FileRef::new(&(unit_test_dir.path().to_owned() + "/subdir2")).create(),
			FileRef::new(&(unit_test_dir.path().to_owned() + "/file1.txt")).create(),
			FileRef::new(&(unit_test_dir.path().to_owned() + "/subdir1/file2.txt")).create(),
			FileRef::new(&(unit_test_dir.path().to_owned() + "/subdir1/sub_subdir1/file3.txt")).create(),
			FileRef::new(&(unit_test_dir.path().to_owned() + "/subdir2/file4.txt")).create()
		];
		unit_test_dir
	}

	#[test]
	fn test_include_self() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_self();
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.contains(&temp_file_ref));
		assert_eq!(results.len(), 1);
	}

	#[test]
	fn test_include_files() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_files();
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.iter().all(|f| !f.is_dir()));
		assert_eq!(results.len(), 1);
	}

	#[test]
	fn test_include_dirs() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_dirs();
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.iter().all(|d| d.is_dir()));
		assert_eq!(results.len(), 2); // subdir1, sub_subdir1, subdir2.
	}

	#[test]
	fn test_include_files_and_dirs() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_files().include_dirs();
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.len() > 0);
		assert!(results.iter().any(|e| e.is_dir()));
		assert!(results.iter().any(|e| !e.is_dir()));
	}

	#[test]
	fn test_filter() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_files().recurse().filter(|f| f.name().ends_with(".txt"));
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.iter().all(|f| f.name().ends_with(".txt")));
		assert_eq!(results.len(), 4);
	}

	#[test]
	fn test_recursion() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_files().recurse();
		let results:Vec<FileRef> = scanner.collect();
		assert_eq!(results.len(), 4); // file1, file2, file3, file4.
	}

	#[test]
	fn test_recurse_filter() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&temp_file_ref).include_files().recurse_filter(|d| d.name() != "subdir1");
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.iter().all(|f| !f.path().contains("subdir1")));
	}

	#[test]
	fn test_root_is_file() {
		let temp_file:TempFile = create_test_structure();
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let scanner:FileScanner = FileScanner::new(&FileRef::new(&(temp_file_ref.path().to_owned() + "file1.txt"))).include_files().include_dirs().recurse();
		let results:Vec<FileRef> = scanner.collect();
		assert!(results.iter().all(|f| f.name().ends_with("1.txt")));
		assert_eq!(results.len(), 0);
	}
}