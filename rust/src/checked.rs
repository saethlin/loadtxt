use crate::{Chunk, RustArray};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn loadtxt_checked<T: lexical::FromBytes + Default + Copy + Send>(
    filename: &str,
    comments: &str,
    skiprows: usize,
) -> Result<RustArray<T>, Box<dyn std::error::Error>> {
    let ncpu = num_cpus::get();

    let file = fs::File::open(filename)?;
    let contents = unsafe { memmap::Mmap::map(&file)? };

    // handle skiprows
    let remaining = contents
        .splitn(skiprows as usize + 1, |byte| *byte == b'\n')
        .last()
        .ok_or(format!(
            "No lines left in file after skipping {} rows",
            skiprows
        ))?;

    let first_line = remaining
        .split(|c| *c == b'\n')
        .skip_while(|line| line.starts_with(comments.as_bytes()))
        .next()
        .ok_or(format!(
            "No lines left in file after skipping {} rows",
            skiprows
        ))?;

    let first_row_columns = first_line
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|chunk| !chunk.is_empty())
        .count();
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
                while remaining[slice_end] != b'\n' {
                    slice_end += 1;
                    if slice_end == remaining.len() - 1 {
                        break;
                    }
                }
                slice_end += 1;
            }

            let mut slice = &remaining[slice_begin..slice_end];
            // Slices will contain their trailing newline separator if one exists
            // This produces an empty line when split on b'\n', so remove it
            if let Some(&b'\n') = slice.last() {
                slice = &slice[..slice.len() - 1];
            }
            let error_flag = error_flag.clone();
            scoped.execute(move || {
                // Cannot use enumerate on rows or these_cols because they must
                // outlive their iterators
                let mut rows = 0;
                let mut data = Vec::with_capacity((approx_rows * first_row_columns * 2) / ncpu);

                for line in slice
                    .split(|byte| *byte == b'\n')
                    .filter(|l| !l.starts_with(comments.as_bytes()))
                {
                    if error_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let mut columns_this_row = 0;
                    line.split(|c| c.is_ascii_whitespace())
                        .filter(|s| !s.is_empty())
                        .for_each(|s| {
                            columns_this_row += 1;
                            match lexical::try_parse(s) {
                                Ok(v) => data.push(v),
                                Err(_) => {
                                    error_flag.store(true, Ordering::Relaxed);
                                    *this_thread_chunk = Err(parse_error(s));
                                }
                            };
                        });

                    // Check if we read the right number of elements in this line
                    if columns_this_row != first_row_columns {
                        error_flag.store(true, Ordering::Relaxed);
                        *this_thread_chunk =
                            Err(row_num_error(first_row_columns, columns_this_row, line));
                    }
                    rows += 1;
                }
                if this_thread_chunk.is_ok() {
                    // If we didn't encounter an error, store the result
                    // Should this use an Option<Result<Chunk>>?
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
        rows += chunk.rows;
    }

    Ok(RustArray {
        data,
        rows,
        columns: first_row_columns,
    })
}

#[inline(never)]
fn parse_error(item: &[u8]) -> String {
    format!("Could not parse \"{}\"", String::from_utf8_lossy(item))
}

#[inline(never)]
fn row_num_error(first_row: usize, this_row: usize, line: &[u8]) -> String {
    format!(
        "Expected {} row(s) based on the first line, \
         but found {} when parsing \"{}\"",
        first_row,
        this_row,
        String::from_utf8_lossy(line)
    )
}
