use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};

#[derive(Default, Clone)]
struct Chunk<T> {
    data: Vec<T>,
    rows: u64,
    error_line: Option<u64>,
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

fn unchecked_internal<T>(filename: &str) -> Option<Vec<T>>
where
    T: Clone + Send + lexical::FromBytes,
{
    let file = fs::File::open(filename).unwrap();
    let bytes = unsafe { memmap::Mmap::map(&file).unwrap() };

    let start = std::time::Instant::now();
    let ncpu = num_cpus::get();

    let chunksize = bytes.len() / ncpu;
    let mut parsed_chunks = vec![Vec::new(); ncpu];

    scoped_threadpool::Pool::new(ncpu as u32).scoped(|scoped| {
        // Break up the bytes into roughly equal-size chunks, but make sure
        // to only slice on whitespace characters
        let mut slice_begin = 0;
        for e in &mut parsed_chunks {
            let mut slice_end = slice_begin + chunksize;
            if slice_end > bytes.len() {
                slice_end = bytes.len();
            } else {
                while !bytes[slice_end].is_ascii_whitespace() {
                    slice_end += 1;
                }
            }

            let slice = &bytes[slice_begin..slice_end];
            scoped.execute(move || {
                *e = slice
                    .split(|x| x.is_ascii_whitespace())
                    .filter(|s| s.len() > 0)
                    .map(|s| lexical::parse(s))
                    .collect::<Vec<T>>();
            });

            slice_begin = slice_end;
        }
    });
    println!("{:?}", start.elapsed());

    let start = std::time::Instant::now();
    let mut data = Vec::with_capacity(parsed_chunks.iter().map(|c| c.len()).sum::<usize>());
    for chunk in parsed_chunks {
        data.extend_from_slice(&chunk);
    }
    println!("{:?}", start.elapsed());

    Some(data)
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_i64_unchecked(
    filename: *const c_char,
    size: *mut u64,
) -> *const i64 {
    let filename = match CStr::from_ptr(filename).to_str() {
        Ok(v) => v,
        Err(_) => return std::ptr::null(),
    };

    match unchecked_internal(filename) {
        Some(output) => {
            *size = output.len() as u64;
            let ptr = output.as_ptr();
            std::mem::forget(output);
            ptr
        }
        None => {
            *size = 0;
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn loadtxt_f64_unchecked(
    filename: *const c_char,
    size: *mut u64,
) -> *const f64 {
    let filename = match CStr::from_ptr(filename).to_str() {
        Ok(v) => v,
        Err(_) => return std::ptr::null(),
    };

    match unchecked_internal(filename) {
        Some(output) => {
            *size = output.len() as u64;
            let ptr = output.as_ptr();
            std::mem::forget(output);
            ptr
        }
        None => {
            *size = 0;
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
