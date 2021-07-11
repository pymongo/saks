//! ## 解释chrono::NaiveDate源码中的719162
//! 719162 is the number of days between 1/1/1970 and 1/1/0001
//! In the Gregorian calendar, there are 477 leap years between 1 and 1970, so 365 * 1969 + 477 = 719162 days
//! https://unix.stackexchange.com/questions/149858/convert-a-number-of-seconds-elapsed-to-date-from-arbitrary-start-date
//! chrono为了更方便的计算日期以及闰年的影响，要将unix时间戳距离1970年1日1日的日数偏移到距离0001年1月1日的日数
//! 719162+1是因为Rust的chrono将去年的12月31日作为第一天，这样下标1就等于1月1日比较方便

const I2C_ADDR: [u8; 4] = [0xc0, 0xc1, 0xc2, 0xc3];
const NUMS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];

/// i2cdetect -y 1
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
