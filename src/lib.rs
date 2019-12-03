use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

mod checked;
//mod simd;
use checked::loadtxt_checked;
use checked::Chunk;

// This could be done in parallel, but is it worth it?
#[no_mangle]
pub unsafe extern "C" fn loadtxt_flatten_chunks(chunks: *mut c_void, output: *mut f64) {
    let chunks: Box<Vec<Chunk<f64>>> = Box::from_raw(chunks as *mut Vec<Chunk<f64>>);
    let mut start = 0isize;
    for chunk in chunks.iter() {
        std::ptr::copy_nonoverlapping(chunk.data.as_ptr(), output.offset(start), chunk.data.len());
        start += chunk.data.len() as isize;
    }
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_get_chunks(
    filename: *const c_char,
    comments: *const c_char,
    skiprows: usize,
    usecols: *const u64,
    n_usecols: usize,
    max_rows_ptr: *const u64,
    rows: *mut usize,
    cols: *mut usize,
    error: *mut *const c_char,
) -> *const c_void {
    *rows = 0;
    *cols = 0;
    *error = std::ptr::null();

    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let comments = CStr::from_ptr(comments).to_str().unwrap();

    let usecols = if n_usecols > 0 {
        Some(std::slice::from_raw_parts(usecols, n_usecols))
    } else {
        None
    };

    let max_rows = if max_rows_ptr.is_null() {
        None
    } else {
        Some(*max_rows_ptr)
    };

    match loadtxt_checked(filename, comments, skiprows, usecols, max_rows) {
        Ok(chunks) => {
            let n_elements = chunks.iter().map(|c| c.data.len()).sum::<usize>();
            if n_elements == 0 {
                return Box::leak(Box::new(Vec::new())) as *const Vec<crate::checked::Chunk<f64>>
                    as *const c_void;
            }
            *rows = chunks.iter().map(|c| c.rows).sum();
            *cols = n_elements / *rows;
            Box::leak(Box::new(chunks)) as *const Vec<crate::checked::Chunk<f64>> as *const c_void
        }
        Err(e) => {
            let error_string = CString::new(e.to_string()).unwrap();
            *error = error_string.as_ptr();
            std::mem::forget(error_string);
            std::ptr::null()
        }
    }
}
