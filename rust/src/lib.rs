use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod checked;

pub struct RustArray<T> {
    pub rows: usize,
    pub columns: usize,
    pub data: Vec<T>,
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut f64, len: usize) {
    use std::{alloc, mem};
    alloc::dealloc(
        ptr as *mut u8,
        alloc::Layout::from_size_align(len * mem::size_of::<f64>(), mem::align_of::<f64>())
            .unwrap(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt(
    filename: *const c_char,
    comments: *const c_char,
    skiprows: usize,
    rows: *mut usize,
    cols: *mut usize,
    error: *mut *const c_char,
) -> *const f64 {
    *rows = 0;
    *cols = 0;
    *error = std::ptr::null();

    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let comments = CStr::from_ptr(comments).to_str().unwrap();

    match checked::loadtxt_checked(filename, comments, skiprows) {
        Ok(arr) => {
            *rows = arr.rows;
            *cols = arr.columns;
            let ptr = arr.data.as_ptr();
            std::mem::forget(arr);
            ptr
        }
        Err(e) => {
            let error_string = CString::new(e.to_string()).unwrap();
            *error = error_string.as_ptr();
            std::mem::forget(error_string);
            std::ptr::null()
        }
    }
}
