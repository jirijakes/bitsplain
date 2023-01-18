pub use bitsplain_btc::*;
pub use bitsplain_core::annotations::Annotations;
use bitsplain_core::binary::Binary;
pub use bitsplain_core::bitcoin;
pub use bitsplain_core::decode::{all_decoders, decode_input};
pub use bitsplain_core::hex;
pub use bitsplain_core::tree::*;
pub use bitsplain_core::value::Value;
pub use bitsplain_core::{format_time, parse};
pub use bitsplain_core::{Candidate, Input};

/// Registers new decoder, defined by parser function, under a specified name.
/// Optionally a condidition, in form of a pattern match, can be added.
#[rustfmt::skip]
macro_rules! decoder {
    (
	title = $title: literal,
	group = $group: literal,
	symbol = $symbol: literal,
	$func: path $(,)?) => {
	decoder!(title = $title, group = $group, symbol = $symbol, $func, _);
    };
    (
	title = $title: literal,
	group = $group: literal,
	symbol = $symbol: literal,
	$func: path,
	$(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        inventory::submit! {
            bitsplain_core::decode::Decoder {
		title: $title,
		group: $group,
		symbol: $symbol,
		decode: |b| {
		    if matches!(b, $( $pattern )|+ $( if $guard )?) {
			$func(bitsplain_core::parse::Annotated::new(b.bytes())).ok().and_then(|(x, _)| {
			    use bitsplain_core::nom::InputLength;
			    if x.input_len() > 0 {
				None
			    } else {
				Some(x.annotations())
			    }
			})
		    } else {
			None
		    }
		}
            }
        }
    };
}

decoder!(
    title = "Bitcoin block header",
    group = "btc",
    symbol = "header",
    bitsplain_btc::block::block_header
);

decoder!(
    title = "Serialized Bitcoin transaction",
    group = "btc",
    symbol = "tx",
    bitsplain_btc::tx::tx
);

decoder!(
    title = "Bitcoin script",
    group = "btc",
    symbol = "script",
    bitsplain_btc::datatypes::script
); // without script_len1

decoder!(
    title = "Lightning Network channel announcement",
    group = "ln",
    symbol = "chan_ann",
    bitsplain_ln::gossip::channel_announcement
);

decoder!(
    title = "Lightning Network channel update",
    group = "ln",
    symbol = "chan_upd",
    bitsplain_ln::gossip::channel_update
);

decoder!(
    title = "Lightning Network node announcement",
    group = "ln",
    symbol = "node_ann",
    bitsplain_ln::gossip::node_announcement
);

decoder!(
    title = "Lightning Network gossip timestamp filter",
    group = "ln",
    symbol = "ts_filter",
    bitsplain_ln::gossip::gossip_timestamp_filter
);

decoder!(
    title = "Lightning Network BOLT 12 offer",
    group = "ln",
    symbol = "bolt12o",
    bitsplain_ln::bolt12::bolt12,
    Binary::Bech32(hrp, _ ) if hrp == "lno",
);

decoder!(
    title = "Lightning Network BOLT 12 invoice request",
    group = "ln",
    symbol = "bolt12r",
    bitsplain_ln::bolt12::bolt12,
    Binary::Bech32(hrp, _ ) if hrp == "lnr",
);

decoder!(
    title = "Lightning Network BOLT 12 invoice",
    group = "ln",
    symbol = "bolt12i",
    bitsplain_ln::bolt12::bolt12,
    Binary::Bech32(hrp, _ ) if hrp == "lni",
);

decoder!(
    title = "BIP-47 payment code",
    group = "btc",
    symbol = "bip47",
    bitsplain_btc::bip47::payment_code,
    Binary::Base58Check(b) if b.first() == Some(&0x47)
);

decoder!(
    title = "Bitcoin transaction prevout",
    group = "btc",
    symbol = "prevout",
    bitsplain_btc::tx::tx_out
);
