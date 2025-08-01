use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

/// Creates a greeting string from the given name.
/// 
/// # Safety
/// 
/// `name` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn greet(name: *const c_char) -> *mut c_char {
    assert!(!name.is_null());
    let c_str = CStr::from_ptr(name);

    let name_str = c_str.to_str().unwrap_or("World");
    let greeting = format!("Hello, {name_str}! From Rust 🦀");

    CString::new(greeting).unwrap().into_raw()
}

/// Frees a string previously allocated by Rust.
/// 
/// # Safety
/// 
/// `s` must be a pointer previously returned from a Rust function that allocated
/// the string, or null. Do not call this function twice on the same pointer.
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}

#[no_mangle]
pub extern "C" fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

/// Sums all elements in the given array.
/// 
/// # Safety
/// 
/// `arr` must be a valid pointer to an array of at least `len` i32 elements,
/// or null if `len` is 0.
#[no_mangle]
pub unsafe extern "C" fn sum_array(arr: *const i32, len: usize) -> i32 {
    if arr.is_null() || len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(arr, len);
    slice.iter().sum()
}
