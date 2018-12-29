use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int};

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
    has_error: *mut u8,
    error_line: *mut u64,
) -> *const f64 {
    *rows = 0;
    *cols = 0;
    *has_error = 0;
    *error_line = 0;

    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let comments = CStr::from_ptr(comments).to_str().unwrap();

    let ncpu = num_cpus::get();

    let contents = fs::read_to_string(filename).unwrap();
    // handle skiprows
    let remaining = contents
        .splitn(skiprows as usize + 1, '\n')
        .last()
        .unwrap()
        .trim();

    let first_line = remaining.lines().next().unwrap();
    let num_cols = first_line.split_whitespace().count();
    *cols = num_cols as u64;
    let approx_rows = remaining.len() / first_line.len();
    let chunksize = remaining.len() / ncpu;

    let mut parsed_chunks = vec![Chunk::default(); ncpu];
    // Divide into chunks for threads
    scoped_threadpool::Pool::new(ncpu as u32).scoped(|scoped| {
        let mut slice_begin = 0;
        for e in &mut parsed_chunks {
            let mut slice_end = slice_begin + chunksize;
            if slice_end > remaining.len() {
                slice_end = remaining.len();
            } else {
                while !remaining.is_char_boundary(slice_end)
                    || remaining.as_bytes()[slice_end] != b'\n'
                {
                    slice_end += 1;
                    if slice_end == remaining.len() {
                        break;
                    }
                }
            }

            let slice = &remaining[slice_begin..slice_end];
            scoped.execute(move || {
                // Cannot use enumerate on rows or these_cols because they must
                // outlive their iterators
                let mut error_line = None;
                let mut rows = 0;
                let mut data = Vec::with_capacity((approx_rows * num_cols * 2) / ncpu);
                slice
                    .trim()
                    .lines()
                    .filter(|l| !l.starts_with(comments))
                    .for_each(|l| {
                        let mut these_cols = 0;
                        l.split_whitespace().for_each(|s| {
                            these_cols += 1;
                            match s.parse() {
                                Ok(v) => data.push(v),
                                Err(_) => {
                                    if error_line.is_none() {
                                        error_line = Some(rows)
                                    }
                                }
                            };
                        });
                        if these_cols != num_cols {
                            if error_line.is_none() {
                                error_line = Some(rows)
                            }
                        }
                        rows += 1;
                    });
                *e = Chunk {
                    data,
                    rows,
                    error_line,
                }
            });

            slice_begin = slice_end;
        }
    });

    let mut data =
        Vec::with_capacity(parsed_chunks.iter().map(|c| c.data.len()).sum::<usize>() + 1);
    for chunk in parsed_chunks {
        data.extend_from_slice(&chunk.data);
        if let Some(line) = chunk.error_line {
            if *has_error == 0 {
                *error_line = line + *rows;
                *has_error = 1;
            }
        }
        *rows += chunk.rows as u64;
    }

    let ptr = data.as_ptr();
    std::mem::forget(data);
    ptr
}

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
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
