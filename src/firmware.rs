
pub struct Firmware<'a> {
    pub addr: u32,
    pub data: &'a [u8],
}

pub const FIRMWARE_1_3_1_to_1_3_2: [u8; 233292] = *include_bytes!("../third_party/firmware/mfw_nrf9160_update_from_1.3.1_to_1.3.2.bin");

include!(concat!(env!("OUT_DIR"), "/FIRMWARE0.rs"));

include!(concat!(env!("OUT_DIR"), "/FIRMWARE1.rs"));