use saks::{Saks, SaksPins, VoltageLevel};

/// **gpio** command was error on raspberrypi 4: `Oops - unable to determine board type... model: 17`
/// use **raspi-gpio** instead, eg. `gpio readall` is same as `raspi-gpio get`
/// NOTE: if buzzer keep beeping after process exit, try `raspi-gpio set 12 dh` to turn off buzzer
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
