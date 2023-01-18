use clap::ValueEnum;
use serde::Deserialize;

use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Detail {
    Short,
    Normal,
    Extra,
    Debug,
}

pub struct BtcUnit;

pub struct NumFmt;

pub struct HexFmt {
    pub max_len: Option<usize>,
    pub append_len: bool,
}

pub struct Fmt {
    pub btcunit: BtcUnit,
    pub num: NumFmt,
    pub hex: HexFmt,
}

pub struct Ctx {
    pub detail: Detail,
    pub format: Fmt,
    pub settings: Settings,
}
