#![allow(unused)]

pub struct Firmware<'a> {
    pub addr: u32,
    pub data: &'a [u8],
}


include!(concat!(env!("OUT_DIR"), "/FIRMWARE0.rs"));

include!(concat!(env!("OUT_DIR"), "/FIRMWARE1.rs"));