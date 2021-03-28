use map::Map;
use std::io::Cursor;

#[test]
fn dx_library() {
    let mut file = Cursor::new(include_bytes!("DX-LIBRARY.MAP"));
    let map = Map::from_reader(&mut file).unwrap();
}

#[test]
fn dx_minidoom() {
    let mut file = Cursor::new(include_bytes!("DX-MINIDOOM.MAP"));
    let map = Map::from_reader(&mut file).unwrap();
}

#[test]
fn dx_oldhouse() {
    let mut file = Cursor::new(include_bytes!("DX-OLDHOUSE.MAP"));
    let map = Map::from_reader(&mut file).unwrap();
}

#[test]
fn ll_sewer() {
    let mut file = Cursor::new(include_bytes!("LL-SEWER.MAP"));
    let map = Map::from_reader(&mut file).unwrap();
}
