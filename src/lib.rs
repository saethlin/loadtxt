use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

mod checked;
mod simd;
use checked::loadtxt_checked;
use checked::Chunk;

#[no_mangle]
pub unsafe extern "C" fn loadtxt_flatten_chunks(chunks: *mut c_void, output: *mut f64) {
    let chunks: Box<Vec<Chunk<f64>>> = Box::from_raw(chunks as *mut Vec<Chunk<f64>>);
    let mut start = 0isize;
    for chunk in chunks.iter() {
        std::ptr::copy_nonoverlapping(chunk.data.as_ptr(), output.offset(start), chunk.data.len());
        start += chunk.data.len() as isize;
    }
    /*
    let mut data = vec![0.0f64; chunks.iter().map(|c| c.data.len()).sum()];
    let mut remaining = &mut data[..];
    pool.scoped(|scope| {
        for chunk in &chunks {
            let (left, right) = remaining.split_at_mut(chunk.data.len());
            scope.execute(move || {
                left.copy_from_slice(&chunk.data);
            });
            remaining = right;
        }
    });
    */
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_get_chunks(
    filename: *const c_char,
    comments: *const c_char,
    skiprows: usize,
    usecols: *const u64,
    n_usecols: usize,
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

    match loadtxt_checked(filename, comments, skiprows, usecols) {
        Ok(chunks) => {
            *rows = chunks.iter().map(|c| c.rows).sum();
            *cols = chunks.iter().map(|c| c.data.len()).sum::<usize>() / *rows;
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
