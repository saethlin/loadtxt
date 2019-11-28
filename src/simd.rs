use core::arch::x86_64::*;

pub trait SimdLinesIter<'a> {
    fn simd_lines(&self) -> SimdLines<'a>;
}

impl<'a> SimdLinesIter<'a> for &'a [u8] {
    fn simd_lines(&self) -> SimdLines<'a> {
        SimdLines { remaining: *self }
    }
}

pub struct SimdLines<'a> {
    remaining: &'a [u8],
}

impl<'a> std::iter::Iterator for SimdLines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.remaining.is_empty() {
            return None;
        }

        let newline = find_newline(self.remaining);
        if newline < self.remaining.len() {
            unsafe {
                let line = &self.remaining.get_unchecked(..newline);
                self.remaining = &self.remaining[newline + 1..];
                Some(line)
            }
        } else {
            unsafe {
                let line = self.remaining;
                self.remaining = &self.remaining.get_unchecked(newline..);
                Some(line)
            }
        }
    }
}

pub fn find_newline(input: &[u8]) -> usize {
    let mut start = 0;
    loop {
        let index = unsafe {
            let newline = _mm256_set1_epi8(b'\n' as i8);
            let data = _mm256_loadu_si256(input[start..].as_ptr() as *const __m256i);
            _mm_tzcnt_32(_mm256_movemask_epi8(_mm256_cmpeq_epi8(data, newline)) as u32) as usize
        };
        if index < 32 {
            break start + index;
        } else {
            start += 32;
            if start >= input.len() {
                break input.len();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines() {
        let input = b"1\n22\n333\n4444\n55555\n666666\nf";
        for line in (&input[..]).simd_lines() {
            println!("{:?}", std::str::from_utf8(line).unwrap());
        }
    }

    #[test]
    fn words() {
        let input = b"1 22 333 4444 55555 666666 f";
        for line in (&input[..]).simd_words() {
            println!("{:?}", std::str::from_utf8(line).unwrap());
        }
    }
}

pub trait SimdWordsIter<'a> {
    fn simd_words(&self) -> SimdWords<'a>;
}

impl<'a> SimdWordsIter<'a> for &'a [u8] {
    fn simd_words(&self) -> SimdWords<'a> {
        SimdWords { remaining: *self }
    }
}

pub struct SimdWords<'a> {
    remaining: &'a [u8],
}

impl<'a> std::iter::Iterator for SimdWords<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.remaining.is_empty() {
            return None;
        }

        let space = find_space(self.remaining);
        if space < self.remaining.len() {
            unsafe {
                let line = &self.remaining.get_unchecked(..space);
                self.remaining = &self.remaining.get_unchecked(space + 1..);
                Some(line)
            }
        } else {
            unsafe {
                let line = self.remaining;
                self.remaining = &self.remaining.get_unchecked(space..);
                Some(line)
            }
        }
    }
}

pub fn find_space(input: &[u8]) -> usize {
    let mut start = 0;
    loop {
        let index = unsafe {
            let space = _mm256_set1_epi8(b' ' as i8);
            let data = _mm256_loadu_si256(input[start..].as_ptr() as *const __m256i);
            _mm_tzcnt_32(_mm256_movemask_epi8(_mm256_cmpeq_epi8(data, space)) as u32) as usize
        };
        if index < 32 {
            break start + index;
        } else {
            start += 32;
            if start >= input.len() {
                break input.len();
            }
        }
    }
}

/*
pub struct SimdIter<'a> {
    input: &'a [u8],
    //state_start: usize,
    //state: u32,
    //numerals_start_at: usize,
}


impl<'a> SimdIter<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        SimdIter {
            input,
            //state_start: 0,
            //state: 0,
            //numerals_start_at: 0,
        }
    }

    pub fn next(&mut self) -> usize {
        unsafe {
            // Constant
            let whitespace = _mm_set1_epi8(b' ' as i8);
            // Load the first batch
            let data = _mm_loadu_si128(self.input.as_ptr() as *const __m128i);
            let is_whitespace = _mm_cmpgt_epi8(data, whitespace);
            let mut right_edges = _mm_movemask_epi8(_mm_andnot_si128(
                _mm_slli_si128(is_whitespace, 1),
                is_whitespace,
            )) as u32;
            let mut left_edges = _mm_movemask_epi8(_mm_andnot_si128(
                _mm_srli_si128(is_whitespace, 1),
                is_whitespace,
            )) as u32;

            while left_edges > 0 {
                println!("{:016b}", left_edges);
                println!("{:016b}", right_edges);
                let end = _mm_tzcnt_32(left_edges) as usize;
                let start = _mm_tzcnt_32(right_edges) as usize;

                println!(
                    "{}",
                    std::str::from_utf8(&self.input[start..end + 1]).unwrap()
                );

                //println!("{}:{}", start, end+1);

                right_edges ^= 1 << start;
                left_edges ^= 1 << end;
            }

            /*
            let left_edges = _mm_andnot_si128(_mm_srli_si128(is_whitespace, 1), is_whitespace);
            let left_edges = _mm_movemask_epi8(left_edges) as u32;
            println!("{:b}", left_edges);
            */
            0
        }
    }
}
*/
