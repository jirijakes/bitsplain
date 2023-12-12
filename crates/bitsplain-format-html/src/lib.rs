use std::io::Write;

use bitsplain::decode::Candidate;
use bitsplain::dsl::Reference;
use bitsplain::output::hexblock::*;
use bitsplain::tree::Leaf;
use bitsplain_format::*;

lazy_static::lazy_static! {
    static ref THEME: Vec<String> =
        vec![
            "#8be9fd".to_string(),
            "#ffb86c".to_string(),
            "#50fa7b".to_string(),
            "#ff79c6".to_string(),
            "#bd93f9".to_string(),
            "#ff5555".to_string(),
            "#f1fa8c".to_string(),
        ];
    static ref THEME_SIZE: usize = THEME.len();
}

pub fn render<W: Write>(candidate: Candidate, ctx: &Ctx, out: &mut W) -> Result<(), FormatError> {
    let html = generate(candidate, ctx);
    Ok(out.write_all(html.as_bytes())?)
}

pub fn generate(candidate: Candidate, ctx: &Ctx) -> String {
    let hexblock = HexBlock::from_candidate(&candidate);
    let background = include_str!("background.base64");
    let html = format!(
        r#"<html>
    <header>
        <style>
body {{
  /* font-family: sans-serif; */
  /* https://bg.siteorigin.com/?color=%23263238&pattern=blackmamba&blend=51&intensity=4&noise=0&invert=0&2x=0 */
  background-image: url(data:image/png;base64,{});
  background-repeat: repeat;
  color: #efefef;
}}

code {{
  font-family: 'DejaVu Sans Mono', monospace;
}}

code.hex {{
  font-size: 16pt;
  line-height: 1.5em;
}}

table.legend {{
  border-spacing: 12px 8px;
}}

table.legend tr td {{
  vertical-align: text-top;
}}

table.legend tr th {{
  text-align: left;
}}

table.legend tr td.type {{
  font-family: 'DejaVu Sans Mono', monospace;
  font-size: .9em;
}}

table.legend tr td.length {{
  text-align: right;
  padding-right: 1em;
}}

table.legend tr td.name {{
  white-space: nowrap;
}}

table.legend tr td.name code {{
  padding: 2px 4px;
}}

table.legend dt {{
  font-variant: all-small-caps;
  font-size: .8em;
  font-family: sans-serif;
  float: left;
  clear: left;
  text-align: right;
  width: 3em;
  opacity: .7;
  vertical-align: baseline;
}}

table.legend dd {{
  margin-left: 3.5em;
  font-family: sans-serif; 
}}

table.legend dd.doc {{
  font-size: .95em;
}}

table.legend dd.splain {{
  font-size: .95em;
  font-style: italic;
}}

table.legend dl {{
    margin: 0;
}}

{}
        </style>
    </header>
    <body>
        <code class="hex">
{}
        </code>
        <h2>{}</h2>
        <table class="legend">
<tr><th>Name</th><th>Type</th><th>Length</th><th>Description</th></tr>
{}
        </table>
    </body>
</html>
"#,
        background,
        make_theme(),
        make_code(&hexblock),
        candidate.decoder.title,
        make_legend(&candidate)
    );

    html
}

fn make_legend(candidate: &Candidate) -> String {
    candidate
        .annotations
        .leaves()
        .iter()
        .map(|&l| make_legend_row(l))
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_legend_row(leaf: &Leaf) -> String {
    let x = leaf.index().map(|i| i % *THEME_SIZE).unwrap_or(1000);
    let desc = format!(
        r#"
<dl>
  <dt>Value</dt><dd class="value">{}</dd>
  {}
  {}
</dl>"#,
        leaf.information().value.preview(),
        leaf.information()
            .doc
            .clone()
            .map(|s| {
                let refs = leaf
                    .information()
                    .refs
                    .iter()
                    .map(|r| match r {
                        Reference::Bip(n) => {
                            format!(r#"<a href="https://bips.xyz/{n}">BIP{n}</a>"#)
                        }
                        Reference::Www(www) => format!(r#"<a href="{www}">WWW</a>"#),
                    })
                    .collect::<Vec<_>>();
                format!(r#"<dt>Doc</dt><dd class="doc">{s} {}</dd>"#, refs.join(" "))
            })
            .unwrap_or_default(),
        leaf.information()
            .splain
            .clone()
            .map(|x| format!(r#"<dt>Splain</dt><dd class="splain">{x}</dd>"#))
            .unwrap_or_default()
    );
    format!(
        r#"<tr><td class="name"><code class="fg{x} bg{x}">{}</code></td><td class="type">{}</td><td class="length">{}</td><td class="description">{}</td></tr>"#,
        leaf.information().label,
        leaf.information()
            .data
            .get("datatype")
            .map(|s| s.as_str())
            .unwrap_or_default(),
        leaf.length().map(|l| l.to_string()).unwrap_or_default(),
        desc
    )
}

fn make_code(hexblock: &HexBlock) -> String {
    hexblock
        .rows()
        .iter()
        .map(make_row)
        .collect::<Vec<_>>()
        .join("<br />\n")
}

fn make_row(row: &Row) -> String {
    row.chunks()
        .iter()
        .map(make_chunk)
        .collect::<Vec<_>>()
        .join("")
}

fn make_chunk(chunk: &Chunk) -> String {
    let x = chunk.index() % *THEME_SIZE;
    format!(r#"<span class="fg{x} bg{x}">{}</span>"#, chunk.content())
}

fn make_theme() -> String {
    use colors_transform::*;

    THEME
        .iter()
        .enumerate()
        .map(|(idx, color)| {
            let bg = Rgb::from_hex_str(color).unwrap();
            let fg = bg.lighten(-40.0).to_css_hex_string();
            let bg = bg.lighten(10.0).to_css_hex_string();

            format!(
                r#"
.fg{idx} {{
  color: {fg};
}}

.bg{idx} {{
  background-color: {bg};
}}
"#
            )
        })
        .collect()
}
