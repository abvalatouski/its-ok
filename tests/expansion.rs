use its_ok::{ok, ok_unchecked};
use std::str;

#[test]
fn expand_ok() {
    let string = ok! {
        let valid_utf8 = b"bytes";
        str::from_utf8(valid_utf8)?
    };
    assert_eq!(string, "bytes");
}

#[test]
#[should_panic]
fn expand_ok_with_err() {
    ok! {
        let invalid_utf8 = &[0xFF, 0xFF, 0xFF, 0xFF];
        str::from_utf8(invalid_utf8)?;
    }
}

#[test]
fn expand_ok_unchecked() {
    let string = ok_unchecked! {
        let valid_utf8 = b"bytes";
        unsafe { str::from_utf8(valid_utf8)? }
    };
    assert_eq!(string, "bytes");
}

#[test]
#[ignore = "undefined behavior"]
fn expand_ok_unchecked_with_err() {
    ok_unchecked! {
        let invalid_utf8 = &[0xFF, 0xFF, 0xFF, 0xFF];
        unsafe { str::from_utf8(invalid_utf8)?; }
    }
}
