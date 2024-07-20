use std::io::{Read, Write};
use std::path::PathBuf;

use bitsplain::decode::{all_decoders, decode_input, input_to_binaries, Input};
use bitsplain_format::*;
use clap::Parser;

use crate::args::*;

mod args;
mod format;

fn main() {
    let args: Args = Args::parse();

    if args.list_decoders {
        all_decoders()
            .iter()
            .enumerate()
            .for_each(|(i, d)| println!("{:#2}. [{}/{}] {}", i + 1, d.group, d.symbol, d.title));
        return;
    }

    let input: Input = args
        .input
        .map(Input::String)
        .or_else(|| args.file.map(read_file).map(|b| Input::Binary(b.into())))
        .unwrap_or_else(|| Input::Binary(read_stdin().into()));

    if args.print_hex {
        input_to_binaries(input).iter().take(1).for_each(|bin| {
            let mut out = std::io::stdout();
            let mut printer = hexyl::PrinterBuilder::new(&mut out).build();
            let _ = printer.print_all::<&[u8]>(bin.as_ref());
        });

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

    let settings = conf.try_deserialize::<Settings>().unwrap();

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
        params: args.params.iter().collect(),
    };

    decode_input(input)
        .into_iter()
        .take(1)
        .for_each(|candidate| {
            let mut output: Box<dyn Write> = {
                if let Some(f) = &args.outfile {
                    Box::new(std::fs::File::create(f).unwrap())
                } else {
                    Box::new(std::io::stdout())
                }
            };

            match args.format {
                // TODO: Figure out what to do with outputs other than stdout for pretty
                Format::Pretty => format::pretty::render(candidate, &ctx),
                Format::Html => {
                    bitsplain_format_html::render(candidate, &ctx, &mut output).unwrap()
                }
                Format::Png => {
                    bitsplain_format_image::render(candidate, &ctx, &mut output).unwrap()
                }
                Format::Json => todo!(),
                Format::Xml => bitsplain::output::xml::tree_to_xml(&candidate),
            }
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
