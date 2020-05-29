use std::ffi::OsString;

use clap_nested::Commander;
use regex::Regex;

static CRATE_NAME: &'static str = clap::crate_name!();
static CRATE_VERSION: &'static str = clap::crate_version!();
static CRATE_DESC: &'static str = clap::crate_description!();
static CRATE_AUTHOR: &'static str = clap::crate_authors!();

pub fn assert_output<T>(
    commander: &Commander<'_, (), T>,
    args: impl IntoIterator<Item = impl Into<OsString> + Clone>,
    out: &str,
    use_stderr: bool,
) {
    assert_result(commander.run_with_args_result(args), out, use_stderr);
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
    let right = right.as_ref().to_owned();
    let right = right.replace("__NAME__", CRATE_NAME);
    let right = right.replace("__VERSION__", CRATE_VERSION);
    let right = right.replace("__DESC__", CRATE_DESC);
    let mut right = right.replace("__AUTHOR__", CRATE_AUTHOR);

    if let Some(name) = std::env::args_os().next() {
        let path = std::path::Path::new(&name);

        if let Some(filename) = path.file_name() {
            if let Some(binary_name) = filename.to_os_string().to_str() {
                right = right.replace("__BIN_NAME__", binary_name);
            }
        }
    }

    let regex = Regex::new("\x1b[^m]*m").unwrap();

    // Strip out any mismatching \r character on Windows that might sneak in on either side
    let left_stripped = left.as_ref().trim().replace("\r", "");
    let right_stripped = right.trim().replace("\r", "");

    let left = regex.replace_all(&left_stripped, "");
    let right = regex.replace_all(&right_stripped, "");

    if left != right {
        /* println!(
            "-->left\
             {}\
             -->right\
             {}\
             --",
            left, right
        ); */

        assert!(false);
    }
}
