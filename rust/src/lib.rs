extern crate rayon;
use rayon::prelude::*;

use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub unsafe extern "C" fn loadtxt(
    filename: *const c_char,
    skiprows: c_int,
    rows: *mut u64,
    cols: *mut u64,
) -> *const f64 {
    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let all_contents = fs::read_to_string(filename).unwrap();

    // Skip rows before trimming whitespace
    let contents = all_contents
        .splitn(skiprows as usize + 1, '\n')
        .last()
        .unwrap()
        .trim();

    *rows = contents.lines().count() as u64;

    let first_line = contents.lines().next().unwrap();
    *cols = first_line.split_whitespace().count() as u64;

    let mut data = Vec::with_capacity((*rows * *cols) as usize);

    data.par_extend(
        rayon::iter::split(contents, |data| {
            let guess = data.len() / 2;
            let additional_jump = data[guess..].find('\n');
            if let Some(i) = additional_jump {
                (&data[..guess + i], Some(&data[guess + i..]))
            } else {
                (data, None)
            }
        }).flat_map(|chunk| chunk.par_split_whitespace().map(|x| x.parse::<f64>().unwrap()))
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
