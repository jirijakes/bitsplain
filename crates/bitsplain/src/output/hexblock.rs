use crate::decode::Candidate;

/// Data structure representing a block with hexadecimal
/// output that can assist with generating line-based outputs.
#[derive(Debug)]
pub struct HexBlock {
    /// Maximal width of each row in bytes. All rows will be
    /// aligned to this amount, except the last one.
    width: usize,

    /// Total length of bytes inside the block.
    len: usize,

    /// Block's rows.
    rows: Vec<Row>,
}

impl Default for HexBlock {
    fn default() -> Self {
        Self {
            width: 32,
            len: Default::default(),
            rows: Default::default(),
        }
    }
}

impl HexBlock {
    /// Creates HexBlock from a decoding candidate.
    pub fn from_candidate(candidate: &Candidate) -> HexBlock {
        let data = candidate.data.to_vec();

        candidate
            .annotations
            .real_leaves()
            .iter()
            .fold(HexBlock::default(), |r, &l| {
                r.add_leave(l.location.index, &data[l.location.range()])
            })
    }

    fn add_leave(self, index: usize, data: &[u8]) -> HexBlock {
        let mut rows = self.rows;

        let mut buf = data;
        let mut new_len = self.len;

        while !buf.is_empty() {
            // let inserted = self.len;

            let available = self.width - new_len % self.width;

            let (current, rest) = buf.split_at(available.min(buf.len()));
            buf = rest;

            let mut block = Chunk {
                content: hex::encode(current),
                index,
                len: current.len(),
                offset: 0,
            };

            match rows.last_mut() {
                // We still have space in the last block.
                Some(r) if r.len() < self.width => {
                    block.offset = r.len();
                    r.chunks.push(block);
                }
                // We need to create new row.
                optrow => {
                    let mut r = match optrow {
                        // Not a first row.
                        Some(r) => Row::with_num(r.num + 1),

                        // First row.
                        None => Row::default(),
                    };
                    block.offset = 0;
                    r.chunks.push(block);
                    rows.push(r);
                }
            };

            new_len += current.len();
        }

        HexBlock {
            width: self.width,
            len: new_len,
            rows,
        }
    }

    pub fn rows(&self) -> &[Row] {
        self.rows.as_ref()
    }
}

#[derive(Debug, Default)]
pub struct Row {
    num: usize,
    chunks: Vec<Chunk>,
}

impl Row {
    fn len(&self) -> usize {
        self.chunks.iter().map(|r| r.len).sum()
    }

    fn with_num(num: usize) -> Row {
        Row {
            num,
            ..Default::default()
        }
    }

    pub fn chunks(&self) -> &[Chunk] {
        self.chunks.as_ref()
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    content: String,
    index: usize,
    len: usize,
    offset: usize,
}

impl Chunk {
    pub fn content(&self) -> &str {
        self.content.as_ref()
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
