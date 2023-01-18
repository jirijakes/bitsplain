use std::io::Read;
use std::path::PathBuf;

use ::pretty::termcolor::{ColorChoice, StandardStream};
use ::pretty::RcDoc;
use bitsplain::*;
use clap::Parser;

use crate::ctx::*;

mod ctx;
mod pretty;
mod settings;

#[derive(Parser, Debug)]
#[command(about = "Decodes Bitcoin-related binary data")]
#[command(
    long_about = "Decodes Bitcoin-related binary data\n\nIf neither INPUT nor FILE is provided, read standard input."
)]
#[command(name = "bitsplain")]
#[command(bin_name = "bitsplain")]
#[command(version, author)]
struct Args {
    /// Value to parse (hex, base64, base58, …)
    input: Option<String>,

    /// Level of details to display
    #[arg(long, short)]
    details: Option<Detail>,

    #[arg(long, default_value = "false")]
    /// Show all known decoders
    list_decoders: bool,

    /// Do not show documentation strings
    #[arg(long)]
    show_doc: Option<bool>,

    /// Show value identifiers (can be used for selecting)
    #[arg(long)]
    show_ids: Option<bool>,

    /// Read data from file
    #[arg(short = 'i', display_order = 0)]
    file: Option<PathBuf>, // / Print only value under PATH
                           // #[clap(long, short = 's', value_name = "PATH", display_order = 100)]
                           // select: Option<Select>,

                           // / Output format
                           // #[clap(
                           //     long,
                           //     short = 'f',
                           //     value_name = "FORMAT",
                           //     display_order = 100,
                           //     default_value = "pretty",
                           //     arg_enum
                           // )]
                           // format: Format,
}

fn main() {
    let args: Args = Args::parse();

    if args.list_decoders {
        bitsplain::decode::all_decoders()
            .iter()
            .enumerate()
            .for_each(|(i, d)| println!("{:#2}. [{}/{}] {}", i + 1, d.group, d.symbol, d.title));
        return;
    }

    let conf_file = dirs::config_dir()
        .expect("Could not find directory with configuration files.")
        .join("bitsplain/config.toml");

    let dark_theme = dirs::config_dir()
        .expect("Could not find directory with configuration files.")
        .join("bitsplain/dark.toml");

    let conf = config::Config::builder()
        .add_source(config::File::from(dark_theme))
        .add_source(config::File::from(conf_file))
        .build()
        .unwrap();

    let settings = conf.try_deserialize::<settings::Settings>().unwrap();

    // println!("{:#?}", settings);

    // if args.format == Format::Pretty {
    //     if let Some(over) = args.show_doc {
    //         settings.format.pretty.doc.show = over;
    //     }
    //     if let Some(over) = args.show_ids {
    //         settings.format.pretty.ids.show = over;
    //     }
    // };

    let ctx = Ctx {
        detail: args.details.or(settings.details).unwrap_or(Detail::Short),
        format: Fmt {
            btcunit: BtcUnit,
            num: NumFmt,
            hex: HexFmt {
                max_len: Some(66),
                append_len: true,
            },
        },
        settings,
    };

    let input: Input = args
        .input
        .map(Input::String)
        .or_else(|| args.file.map(read_file).map(|b| Input::Binary(b.into())))
        .unwrap_or_else(|| Input::Binary(read_stdin().into()));

    bitsplain::decode::decode_input(input)
        .into_iter()
        .take(1)
        .for_each(|a| {
            let header = RcDoc::text(a.decoder.title);
            let doc = a
                .annotations
                .0
                .iter()
                .fold(header, |doc, t| {
                    doc.append(RcDoc::line())
                        .append(RcDoc::as_string("- "))
                        .append(pretty::pretty_tree(t, &ctx))
                })
                .nest(4);

            doc.render_colored(100, StandardStream::stdout(ColorChoice::Auto))
                .unwrap();
        });
}

//TODO: Error handling
fn read_file(path: PathBuf) -> Vec<u8> {
    std::fs::read(path).expect("Could not read data from provided file.")
}

//TODO: Error handling
fn read_stdin() -> Vec<u8> {
    let mut buffer = Vec::new();

    std::io::stdin()
        .read_to_end(&mut buffer)
        .expect("Could not read data from standard input.");

    buffer
}
