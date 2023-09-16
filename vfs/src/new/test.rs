use util::rstring::RString;

use super::*;

#[test]
fn smoke_vpaths() {
	let rootpath = VPathBuf(RString::new("/"));

	for comp in rootpath.components() {
		dbg!(comp);
	}

	let vpath = VPathBuf(RString::new("/my_mod/music/subfolder/song.mid"));
	let fpath = FolderPath::from(vpath.clone());
	let spath = SpacedPath::from(vpath.clone());
	let npath = ShortPath::from(vpath.clone());

	assert_eq!(
		std::borrow::Borrow::<VPath>::borrow(&fpath),
		"/my_mod/music/subfolder"
	);
	assert_eq!(
		std::borrow::Borrow::<VPath>::borrow(&spath),
		"subfolder/song.mid"
	);
	assert_eq!(std::borrow::Borrow::<VPath>::borrow(&npath), "song.mid");
}

#[test]
fn smoke_lookup() {
	let vfs = VirtualFs::default();
	assert_eq!(vfs.lookup("/"), Some(vfs.root()));
}
