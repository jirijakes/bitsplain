use std::collections::HashMap;
use std::io::Write;

use bitsplain::decode::Candidate;
use bitsplain_format::*;
use wkhtmlapp::{ImgApp, ImgFormat, WkhtmlInput};

pub fn render<W: Write>(candidate: Candidate, ctx: &Ctx, out: &mut W) -> Result<(), FormatError> {
    let width: u32 = if let Some(w) = ctx.params.get("width") {
        w.parse().map_err(|_| {
            FormatError::Param(format!(
                "Could not parse 'width' parameter with value '{w}' as an integer"
            ))
        })?
    } else {
        1024
    };

    let zoom: f32 = if let Some(z) = ctx.params.get("zoom") {
        z.parse().map_err(|_| {
            FormatError::Param(format!(
                "Could not parse 'zoom' parameter with value '{z}' as a float"
            ))
        })?
    } else {
        1.0
    };

    let mut html = Vec::new();
    bitsplain_format_html::render(candidate, ctx, &mut html)?;
    let html = String::from_utf8_lossy(&html);

    let mut img_app = ImgApp::new().map_err(|e| FormatError::Other(e.to_string()))?;

    let width = width.to_string();
    let zoom = zoom.to_string();

    let args = HashMap::from([("quiet", "true"), ("width", &width), ("zoom", &zoom)]);

    let path: String = img_app
        .set_format(ImgFormat::Png)
        .map_err(|e| FormatError::Other(e.to_string()))?
        .set_args(args)
        .map_err(|e| FormatError::Other(e.to_string()))?
        .run(WkhtmlInput::Html(&html), "bitsplain")
        .map_err(|e| FormatError::Other(e.to_string()))?;

    let mut file = std::fs::File::open(&path)?;
    std::io::copy(&mut file, out)?;
    Ok(std::fs::remove_file(&path)?)
}
