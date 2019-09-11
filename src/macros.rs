#[macro_export]
macro_rules! file_stem {
    () => {
        ::std::path::Path::new(::std::file!())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
    };
}

#[test]
fn file_stem() {
    assert!(file_stem!() == "macros");
}
