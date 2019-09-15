use std::ffi::OsString;

use clap_nested::Commander;
use regex::Regex;

pub fn assert_output<T>(
    commander: &Commander<'_, (), T>,
    args: impl IntoIterator<Item = impl Into<OsString> + Clone>,
    out: &str,
    use_stderr: bool,
) {
    assert_result(commander.run_with_args(args), out, use_stderr);
}

pub fn assert_result(res: Result<(), clap::Error>, out: &str, use_stderr: bool) {
    let mut buf = Vec::new();
    let err = res.unwrap_err();

    err.write_to(&mut buf).unwrap();

    assert_eq!(err.use_stderr(), use_stderr);
    assert_eq_str(String::from_utf8(buf).unwrap(), out);
}

// Inherited from https://github.com/clap-rs/clap/blob/4dbcb87/clap-test.rs#L10-L30
fn assert_eq_str(left: impl AsRef<str>, right: impl AsRef<str>) {
    let regex = Regex::new("\x1b[^m]*m").unwrap();

    // Strip out any mismatching \r character on Windows that might sneak in on either side
    let left_stripped = left.as_ref().trim().replace("\r", "");
    let right_stripped = right.as_ref().trim().replace("\r", "");

    let left = regex.replace_all(&left_stripped, "");
    let right = regex.replace_all(&right_stripped, "");

    if left != right {
        println!();
        println!("--> left");
        println!("{}", left);
        println!("--> right");
        println!("{}", right);
        println!("--");
        assert!(false);
    }
}
