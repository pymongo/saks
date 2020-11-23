//! SAKS pin map in BCM encoding
//! [SAKS official website pin mapping page](https://shumeipai.nxez.com/swiss-army-knife-shield-for-raspberry-pi)

// LED row port with 74HC595
pub const DS: u8 = 6;
pub const STCP: u8 = 13;
pub const SHCP: u8 = 19;

pub const BUZZER: u8 = 12;

// 4-digit seven segment display port with TM1637
pub const DI: u8 = 25;
pub const CLK: u8 = 29;
