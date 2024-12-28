use crate::FileRef;


	
pub(crate) struct TempFile(pub FileRef);
impl Drop for TempFile {
	fn drop(&mut self) {
		let _ = self.0.delete();
	}
}
pub(crate) const UNIT_TEST_DIR:TempFile = TempFile(FileRef::new_const("target/unit_testing_temp_files/"));