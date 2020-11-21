use rppal::gpio::Gpio;

const BUZZER_PIN_BCM: u8 = 12;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = Gpio::new()?.get(BUZZER_PIN_BCM)?.into_output();

    loop {
        pin.toggle();
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}
