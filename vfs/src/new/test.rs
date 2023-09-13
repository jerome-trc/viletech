use super::*;

#[test]
fn lookup() {
	let vfs = VirtualFs::default();
	assert_eq!(vfs.get("/"), Some(vfs.root()));
}
