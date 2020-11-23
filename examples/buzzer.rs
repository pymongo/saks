use rppal::gpio::Gpio;

extern crate saks;
use saks::pin_map::BUZZER;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = Gpio::new()?.get(BUZZER)?.into_output();

    loop {
        pin.toggle();
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}
