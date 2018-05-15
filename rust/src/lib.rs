extern crate rayon;
use rayon::prelude::*;

use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub unsafe extern "C" fn loadtxt(
    filename: *const c_char,
    skiprows: c_int,
    rows: *mut c_int,
    cols: *mut c_int,
) -> *const f64 {
    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let all_contents = fs::read_to_string(filename).unwrap();

    let contents = all_contents.trim();

    *rows = contents.lines().count() as i32 - skiprows;
    let first_line = contents.lines().skip(skiprows as usize).next().unwrap();
    *cols = first_line.split_whitespace().count() as i32;

    let mut data = Vec::new();
    for line in contents.lines().skip(skiprows as usize) {
        for val in line.split_whitespace() {
            data.push(val.parse().unwrap());
        }
    }

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
