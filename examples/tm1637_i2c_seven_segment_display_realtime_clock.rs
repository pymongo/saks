const I2C_ADDR: [u8; 4] = [0xc0, 0xc1, 0xc2, 0xc3];
const NUMS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let saks_gpio = saks::Saks::new();
    loop {
        let tm = unsafe {
            let mut tm = std::mem::zeroed();
            libc::localtime_r(&libc::time(std::ptr::null_mut()), &mut tm);
            tm
        };
        println!(
            "localtime={:02}:{:02}:{:02}",
            tm.tm_hour, tm.tm_min, tm.tm_sec
        );
        let minute = tm.tm_min as u8;
        let second = tm.tm_sec as u8;
        let data: [u8; 4] = [minute / 10, minute % 10, second / 10, second % 10];
        saks_gpio.i2c_bus_write_command(0x44);
        for i in 0..4 {
            saks_gpio.i2c_bus_start();
            saks_gpio.i2c_bus_write_byte(I2C_ADDR[i]);
            saks_gpio.i2c_bus_write_byte(NUMS[data[i] as usize]);
            saks_gpio.i2c_bus_start();
        }
        saks_gpio.i2c_bus_write_command(0x8f);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
