#[cfg(test)]
mod tests {
	use std::{ sync::Mutex, thread::{ self, sleep }, time::Duration };
	use crate::{ DirMonitor, FileRef };



	#[test]
	fn dir_monitor_full_test() {

		// Prepare temp dir.
		let temp_dir:FileRef = FileRef::new("target/dir_monitor_test");
		if temp_dir.exists() {
			temp_dir.delete().unwrap();
		}
		temp_dir.create().unwrap();

		// Create monitor and run in separate thread.
		static MONITOR_ACTIVE:Mutex<bool> = Mutex::new(true);
		static HISTORY:Mutex<Vec<String>> = Mutex::new(Vec::new());
		let temp_dir_clone:FileRef = temp_dir.clone();
		thread::spawn(move || {
			let monitor:DirMonitor = DirMonitor::new(temp_dir_clone.path())
							.recursive()
							.with_add_handler(|file| HISTORY.lock().unwrap().push(format!("add {}", file.clone())))
							.with_remove_handler(|file| HISTORY.lock().unwrap().push(format!("remove {}", file.clone())))
							.with_modify_handler(|file| HISTORY.lock().unwrap().push(format!("modify {}", file.clone())))
							.with_rename_handler(|origin, file| HISTORY.lock().unwrap().push(format!("rename {} {}", origin.clone(), file.clone())));
			monitor.run_while(|_| *MONITOR_ACTIVE.lock().unwrap()).unwrap();
		});

		// Trigger actions in dir.
		sleep(Duration::from_millis(250));
		(temp_dir.clone() + "/file_a.txt").create().unwrap();
		(temp_dir.clone() + "/file_a.txt").write("T".to_string()).unwrap();
		(temp_dir.clone() + "/file_b.txt").write("T".to_string()).unwrap();
		(temp_dir.clone() + "/file_a.txt").delete().unwrap();
		(temp_dir.clone() + "/subdir").create().unwrap();
		(temp_dir.clone() + "/subdir/file_c.txt").create().unwrap();
		(temp_dir.clone() + "/subdir/file_c.txt").write("T".to_string()).unwrap();
		(temp_dir.clone() + "/subdir/file_c.txt").delete().unwrap();

		// Quit monitor.
		*MONITOR_ACTIVE.lock().unwrap() = false;
		(temp_dir.clone() + "/exit_trigger.txt").create().unwrap();
		sleep(Duration::from_millis(100));

		// Validate correct history.
		const EXPECTED_HISTORY:&[&'static str] = &[
			"add target/dir_monitor_test/file_a.txt",
			"modify target/dir_monitor_test/file_a.txt",
			"add target/dir_monitor_test/file_b.txt",
			"modify target/dir_monitor_test/file_b.txt",
			"remove target/dir_monitor_test/file_a.txt",
			"add target/dir_monitor_test/subdir/file_c.txt",
			"modify target/dir_monitor_test/subdir/file_c.txt",
			"remove target/dir_monitor_test/subdir/file_c.txt",
			"add target/dir_monitor_test/exit_trigger.txt"
		];
		assert_eq!(EXPECTED_HISTORY, *HISTORY.lock().unwrap());

		// Delete temp dir.
		if temp_dir.exists() {
			temp_dir.delete().unwrap();
		}
	}
}