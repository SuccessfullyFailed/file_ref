#[cfg(test)]
mod tests {
	use crate::{ FileRef, unit_test_support::TempFile };
	


	/* PATH TESTS */
	
	#[test]
	fn test_new() {
		let path:&str = "dir\\file.txt";
		let fs_path:FileRef = FileRef::new(path);
		assert_eq!(fs_path.path(), "dir/file.txt");
	}

	#[test]
	fn test_new_const() {
		const PATH:&str = "static/dir/file.txt";
		let fs_path:FileRef = FileRef::new_const(PATH);
		assert_eq!(fs_path.path(), PATH);
	}

	#[test]
	fn test_path() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert_eq!(fs_path.path(), "dir/file.txt");
	}
	
	#[test]
	fn test_messy_path() {
		let path:&str = "./dir1/dir2/..//../file.txt";
		let fs_path:FileRef = FileRef::new(path);
		assert_eq!(fs_path.path(), "file.txt");
	}

	#[test]
	fn test_path_to_absolute() {
		let path:&str = "dir/file.txt";
		let fs_path:FileRef = FileRef::new(path).absolute();
		assert!(fs_path.path().contains(":"), "Did not correctly create absolute path");
	}

	#[test]
	fn test_path_to_relative() {
		let path:String = std::env::current_dir().unwrap().display().to_string() + "/dir/file.txt";
		let fs_path:FileRef = FileRef::new(&path).relative();
		assert_eq!(fs_path.path(), "dir/file.txt");
	}

	#[test]
	fn test_relative_path_to() {
		let path:FileRef = FileRef::new("C:/users/Me/Desktop/file.txt");
		let fs_path:FileRef = FileRef::new("C:/users/Me/Download/cracked_version_of_free_tool/definitely_not_a_virus.exe");
		assert_eq!(path.relative_path_to(&fs_path).path(), "../../Download/cracked_version_of_free_tool/definitely_not_a_virus.exe");
	}

	#[test]
	fn test_parent_dir() {
		let fs_path:FileRef = FileRef::new("dir/subdir/file.txt");
		let parent:FileRef = fs_path.parent_dir().unwrap();
		assert_eq!(parent.path(), "dir/subdir");
	}

	#[test]
	fn test_parent_dir_relative_root() {
		let fs_path:FileRef = FileRef::new("file.txt");
		assert_eq!(fs_path.parent_dir().unwrap(), FileRef::working_dir());
	}

	#[test]
	fn test_parent_dir_absolute_root() {
		let fs_path:FileRef = FileRef::new("C:");
		println!("{:?}", fs_path.parent_dir());
		assert!(fs_path.parent_dir().is_err());
	}

	#[test]
	fn test_parent_dir_ends_with_slash() {
		let fs_path:FileRef = FileRef::new("test1/test2/");
		assert_eq!(fs_path.parent_dir().unwrap().path(), "test1");
	}

	#[test]
	fn test_path_nodes() {
		let fs_path:FileRef = FileRef::new("dir/subdir/file.txt");
		let nodes:Vec<&str> = fs_path.path_nodes();
		assert_eq!(nodes, vec!["dir", "subdir", "file.txt"]);
	}

	#[test]
	fn test_last_node() {
		let fs_path:FileRef = FileRef::new("dir/subdir/file.txt");
		assert_eq!(fs_path.last_node(), "file.txt");
	}

	#[test]
	fn test_len() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert_eq!(fs_path.len(), 12);
	}

	#[test]
	fn test_is_empty() {
		let fs_path:FileRef = FileRef::new("");
		assert!(fs_path.is_empty());

		let fs_path:FileRef = FileRef::new("not_empty");
		assert!(!fs_path.is_empty());
	}

	#[test]
	fn test_contains() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert!(fs_path.contains("file"));
		assert!(!fs_path.contains("no_file"));
	}

	#[test]
	fn test_starts_with() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert!(fs_path.starts_with("dir"));
		assert!(!fs_path.starts_with("file"));
	}

	#[test]
	fn test_ends_with() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert!(fs_path.ends_with("file.txt"));
		assert!(!fs_path.ends_with("dir"));
	}

	#[test]
	fn test_to_lowercase() {
		let fs_path:FileRef = FileRef::new("DIR/FILE.TXT");
		let lower:FileRef = fs_path.to_lowercase();
		assert_eq!(lower.path(), "dir/file.txt");
	}

	#[test]
	fn test_to_uppercase() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let upper:FileRef = fs_path.to_uppercase();
		assert_eq!(upper.path(), "DIR/FILE.TXT");
	}

	#[test]
	fn test_trim() {
		let fs_path:FileRef = FileRef::new("   dir/file.txt   ");
		let trimmed:FileRef = fs_path.trim();
		assert_eq!(trimmed.path(), "dir/file.txt");
	}

	#[test]
	fn test_strip_prefix() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let stripped:Option<FileRef> = fs_path.strip_prefix("dir/");
		assert!(stripped.is_some());
		assert_eq!(stripped.unwrap().path(), "file.txt");
	}

	#[test]
	fn test_strip_suffix() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let stripped:Option<FileRef> = fs_path.strip_suffix(".txt");
		assert!(stripped.is_some());
		assert_eq!(stripped.unwrap().path(), "dir/file");
	}

	#[test]
	fn test_replace() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let replaced:FileRef = fs_path.replace("file", "document");
		assert_eq!(replaced.path(), "dir/document.txt");
	}

	#[test]
	fn test_repeat() {
		let fs_path:FileRef = FileRef::new("file_");
		let repeated:FileRef = fs_path.repeat(3);
		assert_eq!(repeated.path(), "file_file_file_");
	}



	/* FILE MODIFICATION TESTS */

	#[test]
	fn test_file_creation() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		assert!(!temp_file_ref.exists());
		temp_file_ref.create().unwrap();
		assert!(temp_file_ref.exists());
	}

	#[test]
	fn test_file_write_and_read() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content:&str = "Hello, world!";
		temp_file_ref.write(content.to_string()).unwrap();

		let read_content = temp_file_ref.read().unwrap();
		assert_eq!(content, read_content);
	}

	#[test]
	fn test_file_write_bytes_and_read_bytes() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content:&[u8; 20] = b"Hello, binary world!";
		temp_file_ref.write_bytes(content).unwrap();

		let read_content = temp_file_ref.read_bytes().unwrap();
		assert_eq!(content, read_content.as_slice());
	}

	#[test]
	fn test_append_bytes() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		
		temp_file_ref.create().unwrap();

		let initial_content:&str = "Hello";
		let append_content:&str = ", world!";
		temp_file_ref.write(initial_content.to_string()).unwrap();
		temp_file_ref.append_bytes(append_content.as_bytes()).unwrap();

		let read_content = temp_file_ref.read().unwrap();
		assert_eq!(read_content, "Hello, world!");
	}

	#[test]
	fn test_read_range() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content:&str = "Hello, world!";
		temp_file_ref.write(content.to_string()).unwrap();

		let range_content:Vec<u8> = temp_file_ref.read_range(7, 12).unwrap();
		assert_eq!(std::str::from_utf8(&range_content).unwrap(), "world");
	}

	#[test]
	fn test_write_bytes_to_range() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content:&str = "Hello, world!";
		temp_file_ref.write(content.to_string()).unwrap();

		let replacement = "Rust!";
		temp_file_ref.write_bytes_to_range(7, replacement.as_bytes()).unwrap();

		let read_content = temp_file_ref.read().unwrap();
		assert_eq!(read_content, "Hello, Rust!!");
	}

	#[test]
	fn test_file_deletion() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();
		assert!(temp_file_ref.exists());

		temp_file_ref.delete().unwrap();
		assert!(!temp_file_ref.exists());
	}

	#[test]
	fn test_file_copy() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let source_file_ref = temp_file_ref.clone();
		let target_file_ref = temp_file_ref + "_target.txt";

		source_file_ref.create().unwrap();
		let content:&str = "Copy this content.";
		source_file_ref.write(content.to_string()).unwrap();

		source_file_ref.copy_to(&target_file_ref).unwrap();
		assert!(target_file_ref.exists());

		let copied_content = target_file_ref.read().unwrap();
		assert_eq!(content, copied_content);

		target_file_ref.delete().unwrap();
	}
}