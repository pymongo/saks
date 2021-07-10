use saks::{Saks, SaksPins, VoltageLevel};

fn main() {
    let saks_gpio = Saks::new();
    let mut level = VoltageLevel::Low;
    for _ in 0..4 {
        saks_gpio.set_level(SaksPins::Buzzer, level);
        assert_eq!(saks_gpio.get_level(SaksPins::Buzzer), level);
        unsafe {
            libc::sleep(1);
        }
        level = !level;
    }
}
