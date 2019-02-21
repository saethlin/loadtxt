use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicBool, Ordering};

mod checked;

#[derive(Default, Clone)]
struct Chunk<T> {
    data: Vec<T>,
    rows: u64,
    error_line: Option<u64>,
}

struct RustArray<T> {
    rows: u64,
    columns: u64,
    data: Vec<T>,
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt(
    filename: *const c_char,
    comments: *const c_char,
    skiprows: c_int,
    rows: *mut u64,
    cols: *mut u64,
    error: *mut c_char,
) -> *const f64 {
    *rows = 0;
    *cols = 0;
    *error = std::ptr::null();
    *error_line = 0;

    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let comments = CStr::from_ptr(comments).to_str().unwrap();

    let data = checked::loadtxt_checked(filename, comments, skiprows);
    
    match data {
        Ok(d) => {
            let ptr = data.as_ptr();
            std::mem::forget(data);
            ptr
        }
        Err(e) => {
            let error_string = CString::from(e.to_string());
            std::mem::forget(error_string);
            *error = error_string.as_ptr();
            std::ptr::null()
        }
    }
}

use std::io;
fn unchecked_internal<T>(filename: &str) -> io::Result<RustArray<T>>
where
    T: Clone + Send + lexical::FromBytes + Sync + Default,
{
    let ncpu = num_cpus::get();

    let file = fs::File::open(filename)?;
    let input = unsafe { memmap::Mmap::map(&file)? };

    let line_length = input
        .iter()
        .position(|b| *b == b'\n')
        .ok_or(io::Error::new(io::ErrorKind::Other, "No newlines in file"))?
        + 1;
    let num_lines = input.len() / line_length;

    let lines_per_cpu = num_lines / ncpu;
    let bytes_per_cpu = lines_per_cpu * line_length;

    let items_per_line = input[..line_length]
        .split(|x| x.is_ascii_whitespace())
        .filter(|s| s.len() > 0)
        .count();

    let items_per_cpu = (items_per_line * num_lines) / ncpu;

    let mut output = vec![T::default(); items_per_line * num_lines];
    let error_flag = std::sync::Arc::new(AtomicBool::new(false));

    scoped_threadpool::Pool::new(ncpu as u32).scoped(|scoped| {
        for (input_slice, output_slice) in input
            .chunks(bytes_per_cpu)
            .zip(output.chunks_mut(items_per_cpu))
        {
            let error_flag = error_flag.clone();
            scoped.execute(move || {
                let mut number_of_items_parsed = 0;
                for (n, number) in input_slice
                    .split(|x| x.is_ascii_whitespace())
                    .filter(|s| s.len() > 0)
                    .map(|s| lexical::parse(s))
                    .enumerate()
                {
                    match output_slice.get_mut(n) {
                        Some(v) => *v = number,
                        None => {
                            error_flag.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                    number_of_items_parsed += 1;
                }
                if number_of_items_parsed != output_slice.len() {
                    error_flag.store(true, Ordering::Relaxed);
                }
            });
        }
    });

    if error_flag.load(Ordering::Relaxed) {
        Err(io::Error::new(io::ErrorKind::Other, "Error parsing file"))
    } else {
        Ok(RustArray {
            rows: num_lines as u64,
            columns: items_per_line as u64,
            data: output,
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_i64_unchecked(
    filename: *const c_char,
    rows: *mut u64,
    columns: *mut u64,
    error: *mut *const c_char,
) -> *const i64 {
    let filename = match CStr::from_ptr(filename).to_str() {
        Ok(v) => v,
        Err(_) => {
            let error_message = CString::new("Filename must be valid UTF-8").unwrap();
            *error = error_message.as_ptr() as *mut c_char;
            std::mem::forget(error_message);
            return std::ptr::null();
        }
    };

    match unchecked_internal(filename) {
        Ok(output) => {
            *error = std::ptr::null_mut();

            *rows = output.rows;
            *columns = output.columns;
            let ptr = output.data.as_ptr();
            std::mem::forget(output);
            ptr
        }
        Err(e) => {
            let error_message = CString::new(e.to_string()).unwrap();
            *error = error_message.as_ptr() as *mut c_char;
            std::mem::forget(error_message);

            *rows = 0;
            *columns = 0;
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_f64_unchecked(
    filename: *const c_char,
    rows: *mut u64,
    columns: *mut u64,
    error: *mut *const c_char,
) -> *const f64 {
    let filename = match CStr::from_ptr(filename).to_str() {
        Ok(v) => v,
        Err(_) => {
            let error_message = CString::new("Filename must be valid UTF-8").unwrap();
            *error = error_message.as_ptr() as *mut c_char;
            std::mem::forget(error_message);
            return std::ptr::null();
        }
    };

    match unchecked_internal(filename) {
        Ok(output) => {
            *error = std::ptr::null_mut();

            *rows = output.rows;
            *columns = output.columns;
            let ptr = output.data.as_ptr();
            std::mem::forget(output);
            ptr
        }
        Err(e) => {
            let error_message = CString::new(e.to_string()).unwrap();
            *error = error_message.as_ptr() as *mut c_char;
            std::mem::forget(error_message);

            *rows = 0;
            *columns = 0;
            std::ptr::null()
        }
    }
}

/*
fn parse_unchecked(digits: &[u8]) -> i64 {
    let mut result = 0_i64;
    for &c in digits {
        result *= 10;
        result += (c - b'0') as i64;
    }
    result
}
*/
