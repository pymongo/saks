//! 流水灯

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
