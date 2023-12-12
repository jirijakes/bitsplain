mod legend;

use bitsplain::decode::Candidate;
use colors_transform::*;
use svg::node::element::*;
use svg::node::Text as T;
use ttf_parser::Face;

use crate::ctx::Ctx;

const FONT_SIZE: f32 = 16.0;
const HEIGHT: f32 = 1.5;
const VERT: f32 = HEIGHT / 2.0;
const MAX: usize = 31; // bytes

lazy_static::lazy_static! {
    static ref THEME: Vec<Rgb> =
        vec![
            Rgb::from_hex_str("#8be9fd").unwrap(),
            Rgb::from_hex_str("#ffb86c").unwrap(),
            Rgb::from_hex_str("#50fa7b").unwrap(),
            Rgb::from_hex_str("#ff79c6").unwrap(),
            Rgb::from_hex_str("#bd93f9").unwrap(),
            Rgb::from_hex_str("#ff5555").unwrap(),
            Rgb::from_hex_str("#f1fa8c").unwrap(),
        ];
    static ref THEME_SIZE: usize = THEME.len();
}

#[derive(Debug, Default)]
struct Block {
    content: String,
    index: usize,
    len: usize,
    offset: usize,
}

#[derive(Debug, Default)]
struct Row {
    num: usize,
    blocks: Vec<Block>,
}

impl Row {
    fn len(&self) -> usize {
        self.blocks.iter().map(|r| r.len).sum()
    }

    fn with_num(num: usize) -> Row {
        Row {
            num,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct Rows {
    /// Width of rows in bytes. All rows will be
    /// aligned to this amount, except the last one.
    width: usize,

    len: usize,

    /// Rows.
    rows: Vec<Row>,
}

impl Default for Rows {
    fn default() -> Self {
        Self {
            width: MAX,
            len: Default::default(),
            rows: Default::default(),
        }
    }
}

impl Rows {
    fn from_candidate(candidate: &Candidate) -> Rows {
        let data = candidate.data.to_vec();

        candidate
            .annotations
            .leaves()
            .iter()
            .fold(Rows::default(), |r, &l| {
                r.add_leave(l.location.index, &data[l.location.range()])
            })
    }

    fn add_leave(self, index: usize, data: &[u8]) -> Rows {
        let mut rows = self.rows;

        let mut buf = data;
        let mut new_len = self.len;

        while !buf.is_empty() {
            // let inserted = self.len;

            let available = self.width - new_len % self.width;

            let (current, rest) = buf.split_at(available.min(buf.len()));
            buf = rest;

            let mut block = Block {
                content: hex::encode(current),
                index,
                len: current.len(),
                offset: 0,
            };

            match rows.last_mut() {
                // We still have space in the last block.
                Some(r) if r.len() < self.width => {
                    block.offset = r.len();
                    r.blocks.push(block);
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
                    r.blocks.push(block);
                    rows.push(r);
                }
            };

            new_len += current.len();
        }

        Rows {
            width: self.width,
            len: new_len,
            rows,
        }
    }
}

pub fn render(candidate: Candidate, ctx: &Ctx) {
    let font: &[u8] = include_bytes!("../../../../../DejaVuSansMono.ttf");

    let face = Face::parse(font, 0).unwrap();
    let id = face.glyph_index('0').unwrap();
    let adv = face.glyph_hor_advance(id).unwrap();
    let per = face.units_per_em();
    let wid = adv as f32 / per as f32 * FONT_SIZE;

    let doc = svg::Document::new()
        .set("height", 500)
        .set("width", 700)
        .set("viewbox", (0, 0, 700, 500))
        .add(
            Group::new().set("id", "canvas").add(
                Rectangle::new()
                    .set("x", 0)
                    .set("y", 0)
                    .set("width", 700)
                    .set("height", 500)
                    .set("fill", "#ffffff")
                    .set("stroke", "#000000")
                    .set("stroke-width", "0.5"),
            ),
        );

    let rows = Rows::from_candidate(&candidate);

    let x = rows
        .rows
        .iter()
        .flat_map(|r| {
            r.blocks.iter().map(|b| {
                let bg = unsafe { THEME.get_unchecked(b.index % *THEME_SIZE) };
                let fg = bg.lighten(-40.0);
                group(
                    &b.content,
                    &bg.lighten(10.0).to_css_hex_string(),
                    &fg.to_css_hex_string(),
                    wid,
                )
                .set(
                    "transform",
                    format!(
                        "translate({},{})",
                        2.0 * b.offset as f32 * wid,
                        FONT_SIZE * 1.1 * HEIGHT * r.num as f32
                    ),
                )
            })
        })
        .fold(doc, |svg, g| svg.add(g));

    let x = x.add(legend::legend(&candidate).set("transform", "translate(20, 150)"));

    svg::save("output.svg", &x).unwrap();
}

fn group(content: &str, bg: &str, fg: &str, wid: f32) -> Group {
    Group::new()
        .set("font-size", FONT_SIZE)
        .set("font-family", "DejaVu Sans Mono")
        .set("alignment-baseline", "central")
        .add(
            Rectangle::new()
                .set("x", 0)
                .set("y", 0)
                .set("width", content.len() as f32 * wid)
                .set("height", format!("{}em", HEIGHT))
                .set("fill", bg),
        )
        .add(
            Text::new()
                .set("x", 0)
                .set("y", format!("{}em", VERT))
                .set("fill", fg)
                .add(T::new(content)),
        )
}
