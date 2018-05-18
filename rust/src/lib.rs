extern crate rayon;
extern crate num_cpus;
use rayon::prelude::*;

use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub unsafe extern "C" fn loadtxt(
    filename: *const c_char,
    comments: *const c_char,
    skiprows: c_int,
    rows: *mut u64,
    cols: *mut u64,
) -> *const f64 {
    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let comments = CStr::from_ptr(comments).to_str().unwrap();

    let contents = fs::read_to_string(filename).unwrap();

    let mut filtered = String::with_capacity(contents.len());

    let mut num_rows = 0;
    for line in contents
        .lines()
        .skip(skiprows as usize)
        .filter(|l| !l.starts_with(comments))
    {
        filtered.extend(line.chars());
        filtered.push('\n');
        num_rows += 1;
    }
    *rows = num_rows;

    std::mem::drop(contents);

    let first_line = filtered.lines().nth(1).unwrap();
    *cols = first_line.split_whitespace().count() as u64;
    let data: Vec<_> = filtered
        .par_split_whitespace()
        .map(|x| x.parse::<f64>().unwrap())
        .collect();

    assert_eq!(data.len(), (*rows * *cols) as usize);

    let ptr = data.as_ptr();
    std::mem::forget(data);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_flat_f64(filename: *const c_char, size: *mut u64) -> *const f64 {
    let filename = CStr::from_ptr(filename).to_str().unwrap();

    let data: Vec<_> = fs::read_to_string(filename)
        .unwrap()
        .par_split_whitespace()
        .map(|x| x.parse().unwrap())
        .collect();

    *size = data.len() as u64;
    let ptr = data.as_ptr();
    std::mem::forget(data);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_flat_i64(filename: *const c_char, size: *mut u64) -> *const i64 {
    let filename = CStr::from_ptr(filename).to_str().unwrap();

    let data: Vec<_> = std::fs::read(filename)
        .unwrap()
        .par_split(|c| c.is_ascii_whitespace())
        .filter(|x| x.len() > 0)
        .map(|x| std::str::from_utf8_unchecked(x).parse().unwrap())
        .collect();

    *size = data.len() as u64;
    let ptr = data.as_ptr();
    std::mem::forget(data);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_unsafe_i64(filename: *const c_char, size: *mut u64) -> *const i64 {
    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let ncpu = num_cpus::get();

    let bytes = fs::read(filename).unwrap();
    let bytes_ptr = bytes.as_ptr();
    let chunksize = bytes.len() / ncpu;
    let mut handles = Vec::new();

    let mut slice_begin = 0;
    for _ in 0..ncpu {
        let mut slice_end = slice_begin + chunksize;
        if slice_end > bytes.len() {
            slice_end = bytes.len();
        } else {
            while !bytes[slice_end].is_ascii_whitespace() {
                slice_end += 1;
            }
        }

        let tempstring = std::slice::from_raw_parts(
            bytes_ptr.offset(slice_begin as isize),
            slice_end - slice_begin,
        );
        handles.push(std::thread::spawn(move || {
            tempstring
                .split(|x| x.is_ascii_whitespace())
                .filter(|s| s.len() > 0)
                .map(|s| parse_unchecked(s))
                .collect::<Vec<i64>>()
        }));

        slice_begin = slice_end;
    }

    let mut data = Vec::with_capacity(bytes.capacity());
    for handle in handles {
        data.extend_from_slice(&handle.join().unwrap());
    }

    *size = data.len() as u64;
    let ptr = data.as_ptr();
    std::mem::forget(data);
    ptr
}

fn parse_unchecked(digits: &[u8]) -> i64 {
    let mut result = 0_i64;
    for &c in digits {
        result *= 10;
        result += (c - b'0') as i64;
    }
    result
}
