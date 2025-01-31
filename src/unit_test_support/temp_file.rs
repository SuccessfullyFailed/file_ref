use std::{ path::Path, fs::{ create_dir, remove_dir_all, remove_file } };
use std::sync::Mutex;
use rand::prelude::*;



const TEMP_FILE_DIR:&str = "target/unit_test_support/";
static mut RESERVED_FILES:Mutex<Vec<String>> = Mutex::new(Vec::new());



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TempFile(String);
impl TempFile {

	/* CONSTRUCTOR METHODS */

	/// Create a new temp file.
	pub fn new(extension:Option<&str>) -> TempFile {

		// Get lock to assure the creation of the directory and the creating of the file name only happens once at a time.
		let reserved_files:&mut Vec<String> = unsafe { &mut *RESERVED_FILES.lock().unwrap() };

		// Make sure TEMP_FILE_DIR exists.
		let mut tmp_path:String = String::from(".");
		for path_addition in TEMP_FILE_DIR.split('/') {
			tmp_path += &format!("/{path_addition}");
			if !Path::new(&tmp_path).exists() {
				create_dir(&tmp_path).expect(&format!("Could not create '{tmp_path}' for TEMP_FILE_DIR."));
			}
		}

		// Create random file path.
		let mut file:String = Self::random_file(extension);
		while reserved_files.contains(&file) {
			file = Self::random_file(extension);
		}
		reserved_files.push(file.clone());
		TempFile(file)
	}

	/// Generate a random file.
	fn random_file(extension:Option<&str>) -> String {
		TEMP_FILE_DIR.to_owned() + &Self::random_file_name() + &extension.map(|e| ".".to_owned() + e).unwrap_or_default()
	}

	/// Generate a random file name.
	fn random_file_name() -> String {
		const FILE_NAME_CHARS:&str = "abcdefghijklmnopqrstuvwxyz1234567890";
		const FILE_NAME_LENGTH:usize = 32;

		let mut rng:ThreadRng = rand::thread_rng();
		(0..FILE_NAME_LENGTH).map(|_| FILE_NAME_CHARS.chars().choose(&mut rng).unwrap()).collect::<String>()
	}



	/* PROPERTY GETTER METHODS */

	/// Get the path of the file.
	pub fn path(&self) -> &str {
		&self.0
	}
}
impl Drop for TempFile {
	fn drop(&mut self) {
		use crate::FileRef;

		// Delete file.
		let existing:FileRef = FileRef::new(&self.0);
		if existing.exists() {
			existing.delete().expect("Could not delete temp file");
		}

		// Remove from reserved files.
		let reserved_files:&mut Vec<String> = unsafe { &mut *RESERVED_FILES.lock().unwrap() };
		if let Some(index) = reserved_files.iter().position(|entry| entry == &self.0) {
			reserved_files.remove(index);

			// If no reserved files, delete dir.
			if reserved_files.is_empty() {
				remove_dir_all(TEMP_FILE_DIR).expect("Could not delete TEMP_FILE_DIR after all uses.");
			}
		}
	}
}