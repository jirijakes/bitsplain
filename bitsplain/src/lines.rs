use std::{collections::VecDeque, iter::Enumerate, vec::IntoIter};

use crate::tree::RealLeaf;

pub struct Lines<'a> {
    bytes: &'a [u8],
    leaves: Enumerate<IntoIter<&'a RealLeaf>>,
    /// Number of bytes processed so far.x
    processed: usize,
    /// Number of bytes of each line.
    width: usize,
    /// Complete lines waiting to be emitted.
    prefetched: Vec<Vec<Line<'a>>>,
    /// Partial data that has to be processed when `buffer` is empty.
    /// It is guaranteed that it does not form a complete line given `width`.
    leftover: Option<Line<'a>>,
}

impl<'a> Lines<'a> {
    pub fn new(leaves: Vec<&'a RealLeaf>, bytes: &'a [u8]) -> Lines<'a> {
        Lines {
            bytes,
            leaves: leaves.into_iter().enumerate(),
            processed: 0,
            width: 24,
            prefetched: vec![],
            leftover: None,
        }
    }
}

pub struct Line<'a> {
    pub index: usize,
    pub bytes: &'a [u8],
    pub data: &'a RealLeaf,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Vec<Line<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Let's check whether there is a prefetched complete line.
        // If so, emit immediately.
        self.prefetched.pop().or_else(|| {
            let mut buffer: Vec<Line<'_>> = Vec::new();

            loop {
                // If there is a leftover, move it onto the top of buffer,
                // it has to be processed first.
                if let Some(lo) = self.leftover.take() {
                    buffer.push(lo);
                };

                match self.leaves.next() {
                    // No more leaves, emit all remaining elements in buffer.
                    None => {
                        if buffer.is_empty() {
                            return None;
                        } else {
                            return Some(buffer);
                        }
                    }

                    Some((index, data)) => {
                        // Number of bytes still available on this line.
                        let available = self.width - self.processed % self.width;

                        let bytes: &[u8] = &self.bytes[data.location.from..data.location.to + 1];
                        self.processed += bytes.len();

                        if bytes.len() <= available {
                            // Enough space in the current line. We are not emitting
                            // new line immediately.
                            buffer.push(Line { index, bytes, data });
                        } else {
                            // Not enough space in the current line. Emit a line
                            // and prefetch and persist next lines and leftover.
                            let (part, rest) = bytes.split_at(available);
                            buffer.push(Line {
                                index,
                                bytes: part,
                                data,
                            });

                            // Also create all lines from the rest. It may be an incomplete
                            // line or more lines. Ultimately this vector will contain
                            // only complete lines (see next step).
                            let mut prefetched: VecDeque<Line> = rest
                                .chunks(self.width)
                                .rev()
                                .map(|bs| Line {
                                    index,
                                    bytes: bs,
                                    data,
                                })
                                .collect();

                            // If last chunk of the prefetched lines is incomplete,
                            // remove it from the vector and put into leftover.
                            if let Some(l) = prefetched.front() {
                                if l.bytes.len() < self.width {
                                    self.leftover = prefetched.pop_front();
                                }
                            }
                            self.prefetched = prefetched.into_iter().map(|v| vec![v]).collect();
                            return Some(buffer);
                        };
                    }
                };
            }
        })
    }
}
