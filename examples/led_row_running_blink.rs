//! 流水灯

/// other solution: use 74HC138(38译码器) turn on one LED at the same time(既然流水灯同时只亮一个，那么用38译码器能节约引脚)
/// 74HC595, arduino shiftOut API?
fn main() {
    let saks_gpio = saks::Saks::new();
    let mut led_row_state: u8 = 0x01;
    loop {
        println!("state = {:08b}", led_row_state);
        saks_gpio.led_row_write_a_byte(led_row_state);
        led_row_state = led_row_state.rotate_left(1);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
