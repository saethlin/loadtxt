use crate::{Chunk, RustArray};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn loadtxt_checked(
    filename: &str,
    comments: &str,
    skiprows: i32, // TODO: shouldn't be signed
) -> Result<RustArray<f64>, Box<dyn std::error::Error>> {
    let ncpu = num_cpus::get();

    let file = fs::File::open(filename)?;
    let contents = unsafe { memmap::Mmap::map(&file)? };
    let contents = std::str::from_utf8(&contents)?;

    // handle skiprows
    let remaining = contents.splitn(skiprows as usize + 1, '\n').last().unwrap();

    let first_line = remaining.lines().next().unwrap();
    let first_row_columns = first_line.split_whitespace().count();
    let approx_rows = remaining.len() / first_line.len();
    let chunksize = remaining.len() / ncpu;

    // Flag accessible to all threads so they can abort parsing as soon as any
    // thread fails to parse something
    let error_flag = std::sync::Arc::new(AtomicBool::new(false));

    let mut chunks = vec![Ok(Chunk::default()); ncpu];
    // Divide into chunks for threads
    scoped_threadpool::Pool::new(ncpu as u32).scoped(|scoped| {
        let mut slice_begin = 0;
        for this_thread_chunk in &mut chunks {
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
                let mut rows = 0;
                let mut data = Vec::with_capacity((approx_rows * first_row_columns * 2) / ncpu);

                for line in slice.trim().lines().filter(|l| !l.starts_with(comments)) {
                    if error_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let mut columns_this_row = 0;
                    line.split(|c: char| c.is_ascii_whitespace())
                        .filter(|s| !s.is_empty())
                        .for_each(|s| {
                            columns_this_row += 1;
                            match lexical::try_parse(s) {
                                Ok(v) => data.push(v),
                                Err(err) => {
                                    error_flag.store(true, Ordering::Relaxed);
                                    *this_thread_chunk = Err(err.to_string());
                                }
                            };
                        });
                    if columns_this_row != first_row_columns {
                        error_flag.store(true, Ordering::Relaxed);
                        *this_thread_chunk = Err(format!(
                            "Expected {} row(s) based on the first line, \
                             but found {} when parsing \"{}\"",
                            first_row_columns, columns_this_row, line
                        ));
                    }
                    rows += 1;
                }
                if this_thread_chunk.is_ok() {
                    // If we didn't set e to an error already
                    *this_thread_chunk = Ok(Chunk { data, rows })
                }
            });

            slice_begin = slice_end;
        }
    });

    // Early return if there was an error
    let mut parsed_chunks = Vec::new();
    for c in chunks {
        parsed_chunks.push(c?);
    }

    let mut data =
        Vec::with_capacity(parsed_chunks.iter().map(|c| c.data.len()).sum::<usize>() + 1);
    let mut rows = 0;
    for chunk in parsed_chunks {
        data.extend_from_slice(&chunk.data);
        rows += chunk.rows as u64;
    }

    Ok(RustArray {
        data,
        rows,
        columns: first_row_columns as u64,
    })
}
