use bitsplain::decode::Candidate;
use bitsplain::tree::RealLeaf;
use svg::node::element::*;
use svg::node::Text as T;

const FONT_SIZE: f32 = 12.0;

struct Lines {
    lines: Vec<Line>,
    field_width: usize,
}

impl Lines {
    fn from_candidate(candidate: &Candidate) -> Lines {
        let lines = candidate
            .annotations
            .leaves()
            .iter()
            .map(|&l| Line::from_leaf(l))
            .collect::<Vec<_>>();

        let field_width: usize = field_width(&lines);

        println!("{}", field_width);
        
        Lines { lines, field_width }
    }

    fn to_group(&self) -> Group {
        self.lines
            .iter()
            .enumerate()
            .fold(Group::new(), |g, (i, l)| {
                g.add(l.to_group(i, self.field_width))
            })
    }
}

struct Line {
    field: String,
    datatype: Option<String>,
    length: usize,
    description: Option<String>,
}

impl Line {
    fn from_leaf(leaf: &RealLeaf) -> Line {
        Line {
            field: leaf.information.label.clone(),
            datatype: leaf.information.data.get("datatype").cloned(),
            length: leaf.location.to - leaf.location.from,
            description: leaf.information.doc.clone(),
        }
    }

    fn to_group(&self, i: usize, field_width: usize) -> Group {
        Group::new()
            .set(
                "transform",
                format!("translate(0, {})", 30.0 + i as f32 * FONT_SIZE * 1.2),
            )
            .add(
                Text::new()
                    .set("font-family", "DejaVu Sans Mono")
                    .add(T::new(&self.field)),
            )
            .add(
                Text::new()
                    .set(
                        "transform",
                        format!("translate({}, 0)", field_width as f32 * FONT_SIZE),
                    )
                    .set("font-family", "DejaVu Sans")
                    .add(T::new(self.length.to_string())),
            )
    }
}

pub fn legend(candidate: &Candidate) -> Group {
    let g = Group::new().add(
        Text::new()
            .set("font-family", "DejaVu Sans")
            .add(T::new(candidate.decoder.title)),
    );

    let lines = Lines::from_candidate(candidate);

    g.add(lines.to_group())
}

/// Returns number of charactersr of lognest field name.
fn field_width(lines: &[Line]) -> usize {
    lines
        .iter()
        .map(|l| l.field.len())
        .max()
        .unwrap_or_default()
}
