#[link(name = "test_ffi")]
extern "C" {
    pub fn test_ffi_main() -> i32;
}

#[test]
fn test_ffi() {
    let res = unsafe { test_ffi_main() };
    assert_eq!(res, 0, "c tests returned error code {}", res);
}
