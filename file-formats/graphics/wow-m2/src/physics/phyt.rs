use wow_data::prelude::*;
use wow_data::types::MagicStr;
use wow_data_derive::{WowHeaderR, WowHeaderW};

pub const PHYT: MagicStr = *b"TYHP";

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct Phyt {
    pub phyt: u32,
}
