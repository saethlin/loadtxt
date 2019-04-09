use crate::RustArray;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Chunk<T> {
    data: Vec<T>,
    rows: usize,
}

pub fn loadtxt_checked<T: lexical::FromBytes + Default + Copy + Send>(
    filename: &str,
    comments: &str,
    skiprows: usize,
    usecols: Option<&[u64]>,
) -> Result<RustArray<T>, Box<dyn std::error::Error>> {
    let ncpu = num_cpus::get();

    let file = fs::File::open(filename)?;
    if file.metadata()?.len() == 0 {
        return Ok(RustArray::default());
    }
    let contents = unsafe { memmap::Mmap::map(&file)? };

    // handle skiprows
    let remaining = contents
        .splitn(skiprows as usize + 1, |byte| *byte == b'\n')
        .last()
        .unwrap_or(&[]);
    if remaining.is_empty() {
        return Ok(RustArray::default());
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
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|chunk| !chunk.is_empty())
            .count()
    });

    let chunksize = remaining.len() / ncpu;

    // Flag accessible to all threads so they can abort parsing as soon as any
    // thread fails to parse something
    let error_flag = Arc::new(AtomicBool::new(false));

    let mut chunks = vec![Ok(Chunk::default()); ncpu];
    // Divide into chunks for threads
    scoped_threadpool::Pool::new(ncpu as u32).scoped(|scoped| {
        let mut slice_begin = 0;
        for this_thread_chunk in &mut chunks {
            let end_guess = usize::min(remaining.len(), slice_begin + chunksize);
            let slice_end = remaining[end_guess..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|extra| end_guess + extra + 1) // include the newline
                .unwrap_or(remaining.len());

            let mut slice = &remaining[slice_begin..slice_end];

            /*
            if !slice.is_empty() {
                println!(
                    "{:?} {:?}, {} {}",
                    slice[0] as char,
                    slice[slice.len() - 1] as char,
                    slice_begin,
                    slice_end,
                );
            }
            */

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

    // ? to early return if there was an error in parsing
    let chunks = chunks.into_iter().collect::<Result<Vec<_>, _>>()?;

    let mut data = Vec::with_capacity(chunks.iter().map(|c| c.data.len()).sum());
    let mut rows = 0;
    for chunk in chunks {
        data.extend_from_slice(&chunk.data);
        rows += chunk.rows;
    }

    Ok(RustArray {
        data,
        rows,
        columns,
    })
}

fn parse_chunk<T>(
    chunk: &[u8],
    comments: &str,
    error_flag: &AtomicBool,
    required_columns: usize,
    parsed: &mut Vec<T>,
) -> Result<usize, String>
where
    T: lexical::FromBytes,
{
    let mut rows = 0;
    for line in chunk
        .split(|byte| *byte == b'\n')
        .filter(|l| !l.starts_with(comments.as_bytes()))
    {
        if error_flag.load(Ordering::Relaxed) {
            break;
        }
        let columns_this_row = parse_line(line, parsed)?;
        if columns_this_row != required_columns {
            return Err(format!(
                "Expected {} row(s), \
                 but found {} when parsing \"{}\"",
                required_columns,
                columns_this_row,
                String::from_utf8_lossy(line)
            ));
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
    T: lexical::FromBytes,
{
    let mut rows = 0;
    for line in chunk
        .split(|byte| *byte == b'\n')
        .filter(|l| !l.starts_with(comments.as_bytes()))
    {
        if error_flag.load(Ordering::Relaxed) {
            break;
        }

        let columns_this_row = parse_line_usecols(line, usecols, parsed)?;
        if columns_this_row != usecols.len() {
            return Err(format!(
                "Expected {} row(s), \
                 but found {} when parsing \"{}\"",
                usecols.len(),
                columns_this_row,
                String::from_utf8_lossy(line)
            ));
        }
        rows += 1;
    }
    Ok(rows)
}

// Want to count number of iterations
// But break on the first error
fn parse_line<T>(line: &[u8], parsed: &mut Vec<T>) -> Result<usize, String>
where
    T: lexical::FromBytes,
{
    line.split(|c| c.is_ascii_whitespace())
        .filter(|s| s.len() > 0)
        .try_fold(0, |count_parsed, word| {
            lexical::try_parse(word)
                .map(|item| {
                    parsed.push(item);
                    count_parsed + 1
                })
                .map_err(|_| format!("Could not parse \"{}\"", String::from_utf8_lossy(word)))
        })
}

fn parse_line_usecols<T>(line: &[u8], usecols: &[u64], parsed: &mut Vec<T>) -> Result<usize, String>
where
    T: lexical::FromBytes,
{
    let mut next_usecol_index = 0;
    let mut columns = 0;
    for (w, word) in line
        .split(|c| c.is_ascii_whitespace())
        .filter(|s| s.len() > 0)
        .enumerate()
    {
        if usecols[next_usecol_index] as usize == w {
            let item = lexical::try_parse(word)
                .map_err(|_| format!("Could not parse \"{}\"", String::from_utf8_lossy(word)))?;
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
