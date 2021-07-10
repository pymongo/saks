use rppal::gpio::Gpio;
extern crate saks;
use saks::pin_map::{DS, SHCP, STCP};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ds = Gpio::new()?.get(DS)?.into_output();
    let mut shcp = Gpio::new()?.get(SHCP)?.into_output();
    let mut stcp = Gpio::new()?.get(STCP)?.into_output();
    let led_row_state: u8 = 0x01;
    loop {
        // led_row_state = led_row_state.rotate_left(1);
        println!("state = {:8b}", led_row_state);
        // 74HC595 write 8 bit(to combine as a byte)
        for bit_mask in 0..8 {
            if (led_row_state >> bit_mask) & 1 == 1 {
                ds.set_high();
            } else {
                ds.set_low();
            }
            shcp.set_low();
            shcp.set_high();
        }
        // 74HC595 write a byte
        stcp.set_low();
        stcp.set_high();

        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}
