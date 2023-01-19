use bitsplain::bitcoin::blockdata::opcodes::Ordinary::*;
use bitsplain::bitcoin::blockdata::opcodes::{Class, ClassifyContext};
use bitsplain::bitcoin::blockdata::script::*;
use bitsplain::tree::*;
use bitsplain::value::*;
use bitsplain::*;
use human_size::{Byte, SpecificSize};
use pretty::termcolor::*;
use pretty::RcDoc;
use termion::{color, style};
use time::OffsetDateTime;

use crate::ctx::*;

pub fn pretty_tree(t: &Tree, ctx: &Ctx) -> RcDoc<'static, ColorSpec> {
    match t {
        Tree::Group {
            path,
            location,
            information,
            children,
        } => RcDoc::text(format!(
            "{}{}{}{}:",
            style::Bold,
            style::Faint,
            information.label,
            style::Reset
        ))
        .append(if let Some(tag) = information.tags.first() {
            RcDoc::space().append(RcDoc::text(format!(
                "{} {} {}",
                ctx.settings.format.pretty.tag.font,
                tag.label,
                style::Reset
            )))
        } else {
            RcDoc::nil()
        })
        .append(RcDoc::space())
        .append(pretty_value(&information.value, ctx))
        .append(if ctx.detail == Detail::Debug {
            RcDoc::text(format!(
                "          {}{{from={}, to={}, len={}, index_from={}, index_to={}, path={}, data={:?}}}{}",
                color::Fg(color::LightBlack),
                location.byte_from,
                location.byte_to,
                location.byte_to - location.byte_from,
                location.index_from,
                location.index_to,
                path.join("/"),
		information.data,
                style::Reset
            ))
        } else {
            RcDoc::nil()
        })
        .append(RcDoc::hardline())
        .append(RcDoc::intersperse(
            children.iter().map(|v| {
                Some(RcDoc::as_string("-")
                     .append(RcDoc::space())
                     .append(pretty_tree(v, ctx))
                )
            }),
            RcDoc::hardline(),
        ))
        .nest(4),

        Tree::Leaf(Leaf::Real(RealLeaf {
            path,
            location,
            information,
        })) => RcDoc::text(format!(
            "{}{}{}{}",
            style::Bold,
            style::Faint,
            information.label,
            style::Reset
        ))
        .append(RcDoc::as_string(":"))
        .append(RcDoc::space())
        .append(pretty_value(&information.value, ctx))
        .append(if let Some(tag) = information.tags.first() {
            RcDoc::space().append(RcDoc::text(format!(
                "{} {} {}",
                ctx.settings.format.pretty.tag.font,
                tag.label,
                style::Reset
            )))
        } else {
            RcDoc::nil()
        })
        .append(if ctx.detail == Detail::Debug {
            RcDoc::text(format!(
                "          {}{{from={}, to={}, len={}, index={}, path={}, data={:?}}}{}",
                color::Fg(color::LightBlack),
                location.from,
                location.to,
                location.to - location.from,
                location.index,
                path.join("/"),
		information.data,
                style::Reset
            ))
        } else {
            RcDoc::nil()
        })
        .append(pretty_doc(&information.doc, ctx))
        .append(pretty_doc(&information.splain, ctx)),

        Tree::Leaf(Leaf::Virtual(VirtualLeaf { path, information })) => RcDoc::text(format!(
            "{}{}({}){}",
            style::Bold,
            style::Faint,
            information.label,
            style::Reset
        ))
        .append(RcDoc::as_string(":"))
        .append(RcDoc::space())
        .append(pretty_value(&information.value, ctx))
        .append(if ctx.detail == Detail::Debug {
            RcDoc::text(format!(
                "          {}{{path={}, data={:?}}}{}",
                color::Fg(color::LightBlack),
                path.join("/"),
		information.data,
                style::Reset
            ))
        } else {
            RcDoc::nil()
        })
        .append(pretty_doc(&information.doc, ctx))
        .append(pretty_doc(&information.splain, ctx)),
    }
}

fn pretty_value(value: &Value, ctx: &Ctx) -> RcDoc<'static, ColorSpec> {
    match value {
        Value::Num(n) => {
            RcDoc::as_string(n).annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
        }
        Value::Hash(h) => RcDoc::text(format!("{}{}{}", color::Fg(color::Green), h, style::Reset)),
        Value::Bytes(h) if h.is_empty() => {
            RcDoc::text(format!("{}(empty){}", style::Italic, style::Reset))
        }
        Value::Bytes(h) => pretty_hex(h, ctx),
        Value::Nil => RcDoc::nil(),
        Value::Script(s) => pretty_script(s),
        Value::Timestamp(ts) => RcDoc::text(format!(
            "{}{}{}",
            color::Fg(color::Yellow),
            format_time(ts),
            style::Reset
        )),
        Value::Text {
            text,
            foreground,
            background,
        } => {
            let mut f = String::new();
            if let Some([r, g, b]) = foreground {
                f.push_str(&color::Fg(color::Rgb(*r, *g, *b)).to_string());
            };
            if let Some([r, g, b]) = background {
                f.push_str(&color::Bg(color::Rgb(*r, *g, *b)).to_string());
            };
            f.push_str(text);
            f.push_str(style::Reset.as_ref());
            RcDoc::as_string(f)
        }
        Value::Addr(Some(a)) => RcDoc::text(format!("{}{}{}", style::Bold, a, style::Reset)),
        Value::Addr(None) => RcDoc::text(format!("{}(No address){}", style::Italic, style::Reset)),
        Value::Size(s) => RcDoc::as_string(SpecificSize::new(*s as u32, Byte).unwrap()),
        Value::Signature(s) => pretty_hex(&s.serialize_compact(), ctx),
        Value::PublicKey(k) => pretty_hex(&k.to_bytes(), ctx),
        Value::Sat(s) => RcDoc::text(format!(
            "{}{}{}{}",
            color::Fg(color::Rgb(242, 169, 0)),
            style::Bold,
            s.as_str(),
            style::Reset
        )),
        Value::Alt(v1, v2) => pretty_value(v1, ctx)
            .append(RcDoc::text(" ("))
            .append(pretty_value(v2, ctx))
            .append(RcDoc::text(")")),
    }
}

fn pretty_hex(h: &[u8], ctx: &Ctx) -> RcDoc<'static, ColorSpec> {
    if h.len() > 32 {
        RcDoc::hardline()
    } else {
        RcDoc::nil()
    }
    .append(RcDoc::intersperse(
        h.chunks(32).map(|bs| {
            RcDoc::intersperse(
                bs.chunks(8).map(|b| {
                    RcDoc::text(format!(
                        "{}{}{}",
                        ctx.settings.format.pretty.hex.font,
                        hex::encode(b),
                        style::Reset
                    ))
                }),
                RcDoc::space(),
            )
        }),
        RcDoc::hardline(),
    ))
    .nest(4)
}

fn pretty_doc(doc: &Option<String>, ctx: &Ctx) -> RcDoc<'static, ColorSpec> {
    match doc {
        Some(doc) if ctx.settings.format.pretty.doc.show => RcDoc::hardline()
            .append(RcDoc::text(format!(
                "{}{}{}",
                ctx.settings.format.pretty.doc.font,
                doc,
                style::Reset
            )))
            .nest(2),
        _ => RcDoc::nil(),
    }
}

fn pretty_script(s: &Script) -> RcDoc<'static, ColorSpec> {
    let x: Result<Vec<RcDoc<'static, ColorSpec>>, _> = s
        .instructions()
        .scan(false, |is_op_return, r| {
            let x = r.map(|i| match i {
                Instruction::PushBytes(b) if b.is_empty() => RcDoc::as_string("OP_0"),
                Instruction::PushBytes(b) if *is_op_return => pretty_utf8(b),
                Instruction::PushBytes(b) => RcDoc::text(hex::encode(b)),
                Instruction::Op(op) => {
                    let c: Box<dyn color::Color> = match op.classify(ClassifyContext::Legacy) {
                        Class::Ordinary(
                            OP_RIPEMD160
                            | OP_SHA1
                            | OP_SHA256
                            | OP_HASH160
                            | OP_HASH256
                            | OP_CODESEPARATOR
                            | OP_CHECKSIG
                            | OP_CHECKSIGVERIFY
                            | OP_CHECKMULTISIG
                            | OP_CHECKMULTISIGVERIFY,
                        ) => Box::new(color::Red),
                        Class::Ordinary(OP_EQUAL | OP_EQUALVERIFY) => Box::new(color::Blue),
                        Class::ReturnOp => {
                            *is_op_return = true;
                            Box::new(color::Yellow)
                        }
                        _ => Box::new(color::Yellow),
                    };
                    RcDoc::text(format!("{}{}{}", color::Fg(c.as_ref()), op, style::Reset))
                }
            });
            Some(x)
        })
        .collect();
    match x {
        Ok(s) => RcDoc::hardline()
            .append(RcDoc::intersperse(s.into_iter(), RcDoc::hardline()))
            .nest(4),
        Err(_) => RcDoc::as_string("Bad"),
    }
}

fn pretty_utf8(bs: &[u8]) -> RcDoc<'static, ColorSpec> {
    RcDoc::text(
        String::from_utf8_lossy(bs)
            .chars()
            .map(|c| {
                if c.is_alphabetic() || c.is_ascii_graphic() || c.is_ascii_punctuation() {
                    c
                } else {
                    '.'
                }
            })
            .collect::<String>(),
    )
}

fn format_time(time: &OffsetDateTime) -> String {
    time.format(
        &time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap(),
    )
    .unwrap()
}
