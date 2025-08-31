# FileRef 📂

A Rust crate for **safe and ergonomic file handling**.  
FileRef simplifies common file operations like reading, writing, copying, deleting, and scanning directories — all with a clean API.

---

## ✨ Features

- **File references as constants**  
  Create `FileRef` values at compile-time with `new` or `new_const`. `new` will 'clean up' the path, `new_const` will keep it raw, but allow const and static assigning.

- **Simple file operations**  
  Read, write, append, copy, and delete with intuitive methods.

- **Directory traversal**  
  List files, scan recursively, and filter with custom closures.

- **Operator overloading**  
  Use `+` or `+=` to modify file paths easily.

- **Cross-platform support**  
  Unified, safe abstractions over filesystem operations.

---

## 📦 Installation

Add **FileRef** to your `Cargo.toml`:

```toml
[dependencies]
file_ref = { git="https://github.com/SuccessfullyFailed/file_ref" }
```

---

## 🚀 Example

Here’s an example that:

1. Deletes a temporary file.  
2. Creates a new log file in `./logs`.  
3. Scans `./images` for PNGs, skipping subdirectories starting with `_`.  
4. Copies each image to the temp file while logging actions.  
5. Prints the final log.

```rust
use std::error::Error;
use file_ref::FileRef;

const TEMP_FILE:FileRef = FileRef::new_const("./target/temp.png");
const LOGS_DIR:FileRef = FileRef::new_const("./logs");

fn main() -> Result<(), Box<dyn Error>> {

	// 1. Delete temp file if it exists
	TEMP_FILE.delete()?;

	// 2. Find the next available log file
	let mut log_file:FileRef = LOGS_DIR + "/0.log";
	let mut log_file_index:usize = 0;
	while log_file.exists() {
		log_file_index += 1;
		log_file = LOGS_DIR + "/" + &log_file_index.to_string() + ".log";
	}

	// 3. Scan images directory
	let images_dir:FileRef = FileRef::working_dir() + "/images";
	let png_images:Vec<FileRef> = images_dir.scanner()
		.include_files()
		.recurse_filter(|sub_dir| !sub_dir.starts_with("_"))
		.collect();

	// 4. Copy images to TEMP_FILE, log actions
	for image in png_images {
		if image.exists() {
			log_file.append(format!("[INFO] copying {image} to {TEMP_FILE}\n"))?;
			image.copy_to(&TEMP_FILE)?;
		} else {
			log_file.append(format!("[WARNING] file {image} does not exist\n"))?;
		}
	}

	// 5. Print log contents
	let log:String = log_file.read()?;
	println!("[{log_file}]\n{log}");

	Ok(())
}
```

---

## 📝 License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)  
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!  
Feel free to open a PR or start a discussion.

---
