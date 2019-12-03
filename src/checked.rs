use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use lexical_core::FromLexical;

#[derive(Default, Clone)]
pub struct Chunk<T> {
    pub data: Vec<T>,
    pub rows: usize,
}

pub fn loadtxt_checked<T: FromLexical + Default + Copy + Send>(
    filename: &str,
    comments: &str,
    skiprows: usize,
    usecols: Option<&[u64]>,
    max_rows: Option<u64>,
) -> Result<Vec<Chunk<T>>, Box<dyn std::error::Error>> {
    let ncpu = num_cpus::get();

    let file = File::open(filename)?;
    // Cannot mmap empty files
    if file.metadata()?.len() == 0 {
        return Ok(Vec::new());
    }
    let contents = unsafe { memmap::Mmap::map(&file)? };

    // handle skiprows
    let mut remaining = contents
        .splitn(skiprows as usize + 1, |byte| *byte == b'\n')
        .last()
        .unwrap_or(&[]);

    if remaining.is_empty() {
        return Ok(Vec::new());
    }

    // handle max_rows
    if let Some(0) = max_rows {
        // early exit to prevent underflow
        return Ok(Vec::new());
    }

    if let Some(row_limit) = max_rows {
        let mut rows_so_far = 0;
        let mut simd_cursor = 0;
        let mut ignore_next = false;
        'outer: while simd_cursor < remaining.len().saturating_sub(16) {
            unsafe {
                use core::arch::x86_64::*;
                let reg = _mm_loadu_si128(remaining[simd_cursor..].as_ptr() as *const __m128i);
                let mut mask = _mm_movemask_epi8(_mm_cmpeq_epi8(reg, _mm_set1_epi8(b'\n' as i8)));
                while mask != 0 {
                    let mask_index = _mm_tzcnt_32(mask as u32) as usize;
                    if !ignore_next {
                        rows_so_far += 1;
                        ignore_next = false;
                        if rows_so_far == row_limit {
                            remaining = &remaining[..simd_cursor + mask_index];
                            break 'outer;
                        }
                    }
                    if remaining
                        .get(simd_cursor + mask_index + 1..)
                        .map(|s| s.starts_with(comments.as_bytes()))
                        .unwrap_or(false)
                    {
                        ignore_next = true;
                    }
                    mask ^= 1 << mask_index;
                }
            }
            simd_cursor += 16;
        }
        if rows_so_far != row_limit {
            for newline_index in remaining
                .iter()
                .enumerate()
                .skip(simd_cursor)
                .filter_map(|(index, it)| if *it == b'\n' { Some(index) } else { None })
            {
                if !ignore_next {
                    rows_so_far += 1;
                    if rows_so_far == row_limit {
                        remaining = &remaining[..newline_index];
                        break;
                    }
                }
                ignore_next = false;
                if remaining
                    .get(newline_index + 1..)
                    .map(|s| s.starts_with(comments.as_bytes()))
                    .unwrap_or(false)
                {
                    ignore_next = true;
                }
            }
        }
    }

    let first_line = remaining
        .split(|c| *c == b'\n')
        .skip_while(|line| line.starts_with(comments.as_bytes()))
        .next()
        .ok_or(format!(
            "No lines left in file after skipping {} rows",
            skiprows
        ))?;

    let columns = usecols.map(|u| u.len()).unwrap_or_else(|| {
        first_line
            .split(u8::is_ascii_whitespace)
            .filter(|chunk| !chunk.is_empty())
            .count()
    });

    let chunksize = remaining.len() / ncpu;

    // Flag accessible to all threads so they can abort parsing as soon as any
    // thread fails to parse something
    let error_flag = Arc::new(AtomicBool::new(false));

    let mut chunks: Vec<Result<Chunk<T>, _>> = vec![Ok(Chunk::default()); ncpu];
    // Divide into chunks for threads
    let mut pool = scoped_threadpool::Pool::new(ncpu as u32);
    pool.scoped(|scoped| {
        let mut slice_begin = 0;
        for this_thread_chunk in &mut chunks {
            let end_guess = usize::min(remaining.len(), slice_begin + chunksize);
            let slice_end = remaining[end_guess..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|extra| end_guess + extra + 1) // include the newline
                .unwrap_or(remaining.len());

            let mut slice = &remaining[slice_begin..slice_end];

            if slice.last() == Some(&b'\n') {
                slice = &slice[..slice.len() - 1];
            }

            if slice.is_empty() {
                continue;
            }

            let error_flag = Arc::clone(&error_flag);
            scoped.execute(move || {
                let mut data = Vec::with_capacity(64);
                let parse_result = if let Some(usecols) = usecols {
                    parse_chunk_usecols(slice, comments, &error_flag, usecols, &mut data)
                } else {
                    parse_chunk(slice, comments, &error_flag, columns, &mut data)
                };

                match parse_result {
                    Ok(rows) => *this_thread_chunk = Ok(Chunk { data, rows }),
                    Err(e) => *this_thread_chunk = Err(e),
                }
            });

            // Jump over the trailing newline, but avoid running over the end
            slice_begin = slice_end;
        }
    });

    chunks
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|s| s.into())
}

fn parse_chunk<T>(
    chunk: &[u8],
    comments: &str,
    error_flag: &AtomicBool,
    required_columns: usize,
    parsed: &mut Vec<T>,
) -> Result<usize, String>
where
    T: FromLexical,
{
    let mut rows = 0;
    for line in chunk
        .split(|&b| b == b'\n')
        .filter(|l| !l.starts_with(comments.as_bytes()))
    {
        if error_flag.load(Ordering::Relaxed) {
            break;
        }
        let columns_this_row = parse_line(line, parsed)?;
        if columns_this_row != required_columns {
            if required_columns == 1 {
                return Err(format!(
                    "Expected 1 row, \
                     but found {} when parsing \"{}\"",
                    columns_this_row,
                    String::from_utf8_lossy(line)
                ));
            } else {
                return Err(format!(
                    "Expected {} rows, \
                     but found {} when parsing \"{}\"",
                    required_columns,
                    columns_this_row,
                    String::from_utf8_lossy(line)
                ));
            }
        }
        rows += 1;
    }
    Ok(rows)
}

fn parse_chunk_usecols<T>(
    chunk: &[u8],
    comments: &str,
    error_flag: &AtomicBool,
    usecols: &[u64],
    parsed: &mut Vec<T>,
) -> Result<usize, String>
where
    T: FromLexical,
{
    let mut rows = 0;
    for line in chunk
        .split(|&byte| byte == b'\n')
        .filter(|l| !l.starts_with(comments.as_bytes()))
    {
        if error_flag.load(Ordering::Relaxed) {
            break;
        }

        let columns_this_row = parse_line_usecols(line, usecols, parsed)?;
        if columns_this_row != usecols.len() {
            if usecols.len() == 1 {
                return Err(format!(
                    "Expected 1 row, \
                     but found {} when parsing \"{}\"",
                    columns_this_row,
                    String::from_utf8_lossy(line)
                ));
            } else {
                return Err(format!(
                    "Expected {} rows, \
                     but found {} when parsing \"{}\"",
                    usecols.len(),
                    columns_this_row,
                    String::from_utf8_lossy(line)
                ));
            }
        }
        rows += 1;
    }
    Ok(rows)
}

// Want to count number of iterations
// But break on the first error
fn parse_line<T>(line: &[u8], parsed: &mut Vec<T>) -> Result<usize, String>
where
    T: FromLexical,
{
    line.split(u8::is_ascii_whitespace)
        .filter(|s| !s.is_empty())
        .try_fold(0, |count_parsed, word| {
            lexical_core::parse(word)
                .map(|item| {
                    parsed.push(item);
                    count_parsed + 1
                })
                .map_err(|_| {
                    format!(
                        "Could not convert string to float: \"{}\"",
                        String::from_utf8_lossy(word)
                    )
                })
        })
}

fn parse_line_usecols<T>(line: &[u8], usecols: &[u64], parsed: &mut Vec<T>) -> Result<usize, String>
where
    T: FromLexical,
{
    let mut next_usecol_index = 0;
    let mut columns = 0;
    for (w, word) in line
        .split(u8::is_ascii_whitespace)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        if usecols[next_usecol_index] as usize == w {
            let item = lexical_core::parse(word).map_err(|_| {
                format!(
                    "Could not convert string to float: \"{}\"",
                    String::from_utf8_lossy(word)
                )
            })?;
            parsed.push(item);
            columns += 1;
            next_usecol_index += 1;
            if next_usecol_index == usecols.len() {
                break;
            }
        }
    }
    Ok(columns)
}
