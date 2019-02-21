use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::{Chunk, RustArray};

pub fn loadtxt_checked(
    filename: CStr,
    comments: CStr,
    skiprows: u64,
) -> Result<RustArray<f64>, String> {
    let ncpu = num_cpus::get();

    let file = fs::File::open(filename)?;
    let contents = unsafe { memmap::Mmap::map(&file)? };

    // handle skiprows
    let remaining = contents
        .splitn(skiprows as usize + 1, |b| *b == b'\n')
        .last()
        .unwrap();

    let first_line = remaining.lines().next().unwrap();
    let num_cols = first_line.split_whitespace().count();
    let approx_rows = remaining.len() / first_line.len();
    let chunksize = remaining.len() / ncpu;

    let error_flag = std::sync::Arc::new(AtomicBool::new(false));

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
            let error_flag = error_flag.clone();
            scoped.execute(move || {
                // Cannot use enumerate on rows or these_cols because they must
                // outlive their iterators
                let mut error_line = None;
                let mut rows = 0;
                let mut data = Vec::with_capacity((approx_rows * num_cols * 2) / ncpu);
                for l in slice.trim().lines().filter(|l| !l.starts_with(comments)) {
                    if error_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let mut these_cols = 0;
                    l.split_whitespace().for_each(|s| {
                        these_cols += 1;
                        match lexical::try_parse(s) {
                            Ok(v) => data.push(v),
                            Err(_) => {
                                error_flag.store(true, Ordering::Relaxed);
                                if error_line.is_none() {
                                    error_line = Some(rows)
                                }
                            }
                        };
                    });
                    if these_cols != num_cols {
                        error_flag.store(true, Ordering::Relaxed);
                        if error_line.is_none() {
                            error_line = Some(rows)
                        }
                    }
                    rows += 1;
                }
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
    let mut rows = 0;
    for chunk in parsed_chunks {
        data.extend_from_slice(&chunk.data);
        if let Some(line) = chunk.error_line {
            return Err(format!("Parsing failed on line {}", line));
        }
        rows += chunk.rows as u64;
    }

    Ok(RustArray {
        data,
        rows,
        num_cols,
    })
}
