macro_rules! tests {
    ($($test:ident => $file:expr,)+) => {
        $(
            #[test]
            fn $test() {
                let file = include_bytes!($file);
                map::Map::from_slice(file).unwrap();
            }
        )+
    }
}

tests! {
    _se => "maps/_SE.MAP",
    _st => "maps/_ST.MAP",
    _zoo => "maps/_ZOO.MAP",
    dx_library  => "maps/DX-LIBRARY.MAP",
    dx_oldhouse => "maps/DX-OLDHOUSE.MAP",
    dx_minidoom => "maps/DX-MINIDOOM.MAP",
    dx_conam => "maps/DX-CONAM.MAP",
    dx_gameshow => "maps/DX-GAMESHOW.MAP",
    ll_sewer => "maps/LL-SEWER.MAP",
    ll_chuckles => "maps/LL-CHUCKLES.MAP",
    dukedc1 => "maps/DUKEDC1.MAP",
}
