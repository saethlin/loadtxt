use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, Mutex};

use scoped_threadpool::Pool;

mod inner;
use inner::{loadtxt, Chunk};

lazy_static::lazy_static! {
    pub static ref POOL: Arc<Mutex<Option<Pool>>> = Arc::new(Mutex::new(Some(Pool::new(num_cpus::get() as u32))));
}

fn flatten_chunks<T: Copy + Send + Sync>(chunks: &[Chunk<T>], output: &mut [T]) {
    let mut remaining = &mut output[..]; // Copy the slice object so that the original stays alive for the whole function

    let mut pool = crate::POOL
        .try_lock()
        .ok()
        .and_then(|mut guard| guard.take())
        .unwrap_or_else(|| scoped_threadpool::Pool::new(num_cpus::get() as u32));

    pool.scoped(|scope| {
        for chunk in chunks.into_iter() {
            let (this, rem) = remaining.split_at_mut(chunk.data.len());
            remaining = rem;
            scope.execute(move || {
                this.copy_from_slice(&chunk.data[..]);
            });
        }
    });

    if let Ok(mut guard) = crate::POOL.try_lock() {
        guard.replace(pool);
    }
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_flatten_chunks_f64(chunks: *mut c_void, output: *mut f64) {
    let chunks: Box<Vec<Chunk<f64>>> = Box::from_raw(chunks as *mut Vec<Chunk<f64>>);
    let num_numbers = chunks.iter().map(|c| c.data.len()).sum::<usize>();
    let output = std::slice::from_raw_parts_mut(output, num_numbers);
    flatten_chunks(&chunks, output);
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_flatten_chunks_i64(chunks: *mut c_void, output: *mut i64) {
    let chunks: Box<Vec<Chunk<i64>>> = Box::from_raw(chunks as *mut Vec<Chunk<i64>>);
    let num_numbers = chunks.iter().map(|c| c.data.len()).sum::<usize>();
    let output = std::slice::from_raw_parts_mut(output, num_numbers);
    flatten_chunks(&chunks, output);
}

unsafe fn loadtxt_get_chunks<T: Default + lexical_core::FromLexical>(
    filename: *const u8,
    filename_len: usize,
    comments: *const u8,
    comments_len: usize,
    skiprows: usize,
    usecols: *const u64,
    usecols_len: usize,
    max_rows_ptr: *const u64,
    rows: *mut usize,
    cols: *mut usize,
    error: *mut *const c_char,
) -> *const c_void {
    *rows = 0;
    *cols = 0;
    *error = std::ptr::null();

    let filename = std::str::from_utf8(std::slice::from_raw_parts(filename, filename_len)).unwrap();
    let comments = std::slice::from_raw_parts(comments, comments_len);

    let usecols = if usecols_len == 0 {
        None
    } else {
        Some(std::slice::from_raw_parts(usecols, usecols_len))
    };

    let max_rows = if max_rows_ptr.is_null() {
        None
    } else {
        Some(*max_rows_ptr)
    };

    match loadtxt(filename, comments, skiprows, usecols, max_rows) {
        Ok(chunks) => {
            let n_elements = chunks.iter().map(|c| c.data.len()).sum::<usize>();
            if n_elements == 0 {
                // Can't return a null ptr because that indicates there was an error
                // Grumble grumble that was a bad idea, should fix it
                return Box::leak(Box::new(Vec::new())) as *const Vec<Chunk<T>> as *const c_void;
            }
            *rows = chunks.iter().map(|c| c.rows).sum();
            *cols = n_elements / *rows;
            Box::leak(Box::new(chunks)) as *const Vec<Chunk<T>> as *const c_void
        }
        Err(e) => {
            let error_string = CString::new(e.to_string()).unwrap();
            *error = error_string.as_ptr();
            std::mem::forget(error_string);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_get_chunks_f64(
    filename: *const u8,
    filename_len: usize,
    comments: *const u8,
    comments_len: usize,
    skiprows: usize,
    usecols: *const u64,
    usecols_len: usize,
    max_rows_ptr: *const u64,
    rows: *mut usize,
    cols: *mut usize,
    error: *mut *const c_char,
) -> *const c_void {
    loadtxt_get_chunks::<f64>(
        filename,
        filename_len,
        comments,
        comments_len,
        skiprows,
        usecols,
        usecols_len,
        max_rows_ptr,
        rows,
        cols,
        error,
    )
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_get_chunks_i64(
    filename: *const u8,
    filename_len: usize,
    comments: *const u8,
    comments_len: usize,
    skiprows: usize,
    usecols: *const u64,
    usecols_len: usize,
    max_rows_ptr: *const u64,
    rows: *mut usize,
    cols: *mut usize,
    error: *mut *const c_char,
) -> *const c_void {
    loadtxt_get_chunks::<i64>(
        filename,
        filename_len,
        comments,
        comments_len,
        skiprows,
        usecols,
        usecols_len,
        max_rows_ptr,
        rows,
        cols,
        error,
    )
}
