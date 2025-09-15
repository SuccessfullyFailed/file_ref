use std::{ error::Error, ffi::OsStr, iter::once, os::windows::ffi::OsStrExt, ptr::null_mut };
use crate::FileRef;
use winapi::{
	um::{
		winnt::{ FILE_LIST_DIRECTORY, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE, FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_CREATION, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_INFORMATION },
		winbase::{ FILE_FLAG_BACKUP_SEMANTICS, ReadDirectoryChangesW },
		handleapi::INVALID_HANDLE_VALUE,
		fileapi::CreateFileW
	},
	shared::minwindef::{ DWORD, TRUE, FALSE },
	ctypes::c_void
};



pub struct DirMonitor {
	dir:FileRef,
	recursive:bool,

	on_add_file:Vec<Box<dyn Fn(&FileRef)>>,
	on_remove_file:Vec<Box<dyn Fn(&FileRef)>>,
	on_modify_file:Vec<Box<dyn Fn(&FileRef)>>,
	on_rename_file:Vec<Box<dyn Fn(&FileRef, &FileRef)>>
}
impl DirMonitor {

	/* CONSTRUCTOR METHODS */

	/// Create a new directory monitor.
	pub fn new(path:&str) -> DirMonitor {
		DirMonitor {
			dir: FileRef::new(path),
			recursive: false,

			on_add_file: Vec::new(),
			on_remove_file: Vec::new(),
			on_modify_file: Vec::new(),
			on_rename_file: Vec::new()
		}
	}

	/// Return self with recursive monitoring enabled, making it also trigger on changes in subfolders.
	pub fn recursive(mut self) -> Self {
		self.recursive = true;
		self
	}

	/// Return self with an 'on_add' event handler. Triggers the given function whenever a file is created with the new file as argument.
	pub fn with_add_handler<T:Fn(&FileRef) + 'static>(mut self, handler:T) -> Self {
		self.on_add_file.push(Box::new(handler));
		self
	}

	/// Return self with an 'on_remove' event handler. Triggers the given function whenever a file is removed with the now nonexistent file as argument.
	pub fn with_remove_handler<T:Fn(&FileRef) + 'static>(mut self, handler:T) -> Self {
		self.on_remove_file.push(Box::new(handler));
		self
	}

	/// Return self with an 'on_modify' event handler. Triggers the given function whenever a file is modified with the file as argument.
	pub fn with_modify_handler<T:Fn(&FileRef) + 'static>(mut self, handler:T) -> Self {
		self.on_modify_file.push(Box::new(handler));
		self
	}

	/// Return self with an 'on_rename' event handler. Triggers the given function whenever a file is modified with the old filepath and new filepath as argument.
	pub fn with_rename_handler<T:Fn(&FileRef, &FileRef) + 'static>(mut self, handler:T) -> Self {
		self.on_rename_file.push(Box::new(handler));
		self
	}



	/* USAGE METHODS */

	/// Run forever, activating assigned handlers whenever an action is executed on the directory.
	pub fn run(&self) -> Result<(), Box<dyn Error>> {
		self.run_while(|_| true)
	}

	/// Run while the condition returns true. The condition gets the monitor's directory as argument and is only checked after a file modification. Keeps activating assigned handlers whenever an action is executed on the directory. 
	pub fn run_while<T:Fn(&FileRef) -> bool>(&self, condition:T) -> Result<(), Box<dyn Error>> {

		// Validate dir exists.
		if !self.dir.exists() {
			return Err(format!("Cannot monitor dir '{}' as it does not exist.", self.dir).into());
		}
		let path:Vec<u16> = OsStr::new(self.dir.path()).encode_wide().chain(once(0)).collect();

		unsafe {

			// Get a handle to the directory.
			let target_dir_ptr:*mut winapi::ctypes::c_void = CreateFileW(path.as_ptr(), FILE_LIST_DIRECTORY, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, null_mut(), 3, FILE_FLAG_BACKUP_SEMANTICS, null_mut());
			if target_dir_ptr == INVALID_HANDLE_VALUE {
				return Err(format!("Failed to open directory '{}'.", self.dir).into());
			}

			// Repeatedly listen for actions in the directory.
			let mut buffer:[u8; 1024] = [0u8; 1024];
			while condition(&self.dir) {

				// Try to capture a directory action.
				let mut bytes_returned:DWORD = 0;
				if !self.read_dir_changes(target_dir_ptr, &mut buffer, &mut bytes_returned) {
					return Err("ReadDirectoryChangesW failed.".into());
				}

				// Iterate through file-notify-information in the action.
				let mut offset:usize = 0;
				let mut file_moving_origin:FileRef = FileRef::new("");
				loop {
					let fni:&FILE_NOTIFY_INFORMATION = &*(buffer.as_ptr().add(offset as usize) as *const FILE_NOTIFY_INFORMATION);

					// Build file path from file-notify-information.
					let filename_len:usize = (fni.FileNameLength / 2) as usize;
					let filename:Vec<u16> = std::slice::from_raw_parts(fni.FileName.as_ptr(), filename_len).to_vec();
					let filename:String = String::from_utf16_lossy(&filename);
					let file:FileRef = self.dir.clone() + "/" + &filename;

					// Execute handlers according to action type.
					match fni.Action {
						1 => self.on_add_file.iter().for_each(|handler| handler(&file)),
						2 => self.on_remove_file.iter().for_each(|handler| handler(&file)),
						3 => self.on_modify_file.iter().for_each(|handler| handler(&file)),
						4 => file_moving_origin = file,
						5 => self.on_rename_file.iter().for_each(|handler| handler(&file_moving_origin, &file)),
						_ => {},
					}

					// Move on to next information or break the loop.
					if fni.NextEntryOffset == 0 {
						break;
					}
					offset += fni.NextEntryOffset as usize;
				}
			}
		}

		// Return success.
		Ok(())
	}

	/// Read directory changes once. Keeps the thread until a change is made. Returns false if something went wrong.
	fn read_dir_changes(&self, target_dir_ptr:*mut c_void, buffer:&mut [u8; 1024], bytes_returned:&mut DWORD) -> bool {
		unsafe {
			ReadDirectoryChangesW(
				target_dir_ptr,
				(*buffer).as_mut_ptr() as *mut _,
				buffer.len() as DWORD,
				if self.recursive { TRUE } else { FALSE },
				FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CREATION | FILE_NOTIFY_CHANGE_LAST_WRITE,
				bytes_returned,
				null_mut(),
				None
			) != 0
		}
	}
}