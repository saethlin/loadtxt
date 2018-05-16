extern crate rayon;
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

    std::mem::drop(contents);

    let first_line = filtered.lines().next().unwrap();
    *cols = first_line.split_whitespace().count() as u64;

    *rows = num_rows as u64;
    let mut data = Vec::with_capacity((*rows * *cols) as usize + 1);

    data.par_extend(
        filtered
            .par_split_whitespace()
            .map(|x| x.parse::<f64>().unwrap()),
    );

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
