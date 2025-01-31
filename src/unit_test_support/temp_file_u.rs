#[cfg(test)]
mod tests {
	use std::{ path::Path, fs::File };
	use crate::TempFile;



	#[test]
	fn test_temp_file() {

		// Temp file should not exist on definition.
		let temp_file:TempFile = TempFile::new(None);
		assert!(!Path::new(temp_file.path()).exists(), "Temp file should not exist on definition.");

		// Temp file should be deleted on drop.
		File::create(temp_file.path()).unwrap();
		assert!(Path::new(temp_file.path()).exists(), "Temp file should exist after create.");
		let temp_file_path:String = temp_file.path().to_owned();
		drop(temp_file);
		assert!(!Path::new(&temp_file_path).exists(), "Temp file should not exist after drop.");
	}

	#[test]
	fn test_temp_files_can_be_created_async() {

		// Temp file should be able to be created. If it cannot be, it means it clashes with the other unit tests.
		let _temp_file:TempFile = TempFile::new(None);
	}

	#[test]
	fn test_temp_file_extension() {
		assert!(TempFile::new(Some("txt")).path().ends_with("txt"), "Temp file does not have correct extension.");
		assert!(TempFile::new(Some("png")).path().ends_with("png"), "Temp file does not have correct extension.");
	}
}