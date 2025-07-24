use std::sync::Mutex;
use crate::FileRef;



const TEMP_FILE_DIR:&str = "target/unit_test_support/";
static RESERVED_FILES:Mutex<Vec<FileRef>> = Mutex::new(Vec::new());
static RESERVED_FILE_INDEX:Mutex<usize> = Mutex::new(0);



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TempFile(FileRef);
impl TempFile {

	/* CONSTRUCTOR METHODS */

	/// Create a new temp file.
	pub fn new(extension:Option<&str>) -> TempFile {

		// Get lock to assure the creation of the directory and the creating of the file name only happens once at a time.
		let reserved_files:&mut Vec<FileRef> = &mut *RESERVED_FILES.lock().unwrap();

		// Make sure TEMP_FILE_DIR exists.
		let mut tmp_path:String = String::from(".");
		for path_addition in TEMP_FILE_DIR.split('/') {
			tmp_path += &format!("/{path_addition}");
			let file:FileRef = FileRef::new(&tmp_path);
			if !file.exists() {
				file.create().expect(&format!("Could not create '{tmp_path}' for TEMP_FILE_DIR."));
			}
		}

		// Create random file path.
		let mut file:FileRef = Self::random_file(extension);
		while reserved_files.contains(&file) {
			file = Self::random_file(extension);
		}
		reserved_files.push(file.clone());
		TempFile(file)
	}

	/// Generate a random file.
	fn random_file(extension:Option<&str>) -> FileRef {
		FileRef::new(&(TEMP_FILE_DIR.to_owned() + &Self::get_file_name() + &extension.map(|e| ".".to_owned() + e).unwrap_or_default()))
	}

	/// Generate a random file name.
	fn get_file_name() -> String {
		format!("{:#08}", {
			let id_fetcher = &mut *RESERVED_FILE_INDEX.lock().unwrap();
			*id_fetcher += 1;
			*id_fetcher
		})
	}



	/* PROPERTY GETTER METHODS */

	/// Get the path of the file.
	pub fn path(&self) -> &str {
		&self.0.path()
	}
}
impl Drop for TempFile {
	fn drop(&mut self) {

		// Delete file.
		let existing:&FileRef = &self.0;
		if existing.exists() {
			existing.delete().expect("Could not delete temp file");
		}

		// Remove from reserved files.
		let reserved_files:&mut Vec<FileRef> = &mut *RESERVED_FILES.lock().unwrap();
		if let Some(index) = reserved_files.iter().position(|entry| entry == &self.0) {
			reserved_files.remove(index);

			// If no reserved files, delete dir.
			if reserved_files.is_empty() {
				FileRef::new(TEMP_FILE_DIR).delete().expect("Could not delete TEMP_FILE_DIR after all uses.");
			}
		}
	}
}