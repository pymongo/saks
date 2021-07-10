use libc::time_t;
use rppal::gpio::{Gpio, OutputPin};
struct Tm1637 {
    di: OutputPin,
    clk: OutputPin,
}

const I2C_ADDR: [u8; 4] = [0xc0, 0xc1, 0xc2, 0xc3];
const NUMS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];

impl Tm1637 {
    fn i2c_bus_start(&mut self) {
        self.clk.set_high();
        self.di.set_high();
        self.i2c_bus_delay();
        self.di.set_low();
        self.i2c_bus_delay();
        self.clk.set_low();
        self.i2c_bus_delay();
    }

    fn i2c_bus_delay(&self) {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    fn i2c_bus_write_byte(&mut self, byte: u8) {
        for bit_mask in 0..8 {
            let di_level = (byte >> bit_mask) & 1 == 1;
            self.clk.set_low();
            self.i2c_bus_delay();
            if di_level {
                self.di.set_high();
            } else {
                self.di.set_low();
            }
            self.i2c_bus_delay();
            self.clk.set_high();
            self.i2c_bus_delay();
        }

        self.clk.set_low();
        self.i2c_bus_delay();
        self.di.set_high();
        self.i2c_bus_delay();
        self.clk.set_high();
        self.i2c_bus_delay();
    }

    fn i2c_bus_write_command(&mut self, command: u8) {
        self.i2c_bus_start();
        self.i2c_bus_write_byte(command);
        self.i2c_bus_start();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ic_tm1637 = Tm1637 {
        di: Gpio::new()?.get(25)?.into_output(),
        clk: Gpio::new()?.get(5)?.into_output(),
    };
    let mut tm_struct: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        let time_zero: time_t = 0;
        if libc::localtime_r(&time_zero as *const time_t, &mut tm_struct).is_null() {
            panic!("error in localtime_r system call");
        }
    }
    loop {
        let mut now_sec: time_t = unsafe { std::mem::zeroed() };
        unsafe {
            libc::time(&mut now_sec as *mut time_t);
        }
        now_sec += tm_struct.tm_gmtoff as time_t;
        let today_seconds = now_sec % (24 * 3600);
        let minutes = today_seconds / 60;
        let second = (today_seconds - minutes * 60) as u8;
        let hour = minutes / 60;
        let minute = (minutes - hour * 60) as u8;
        println!("localtime={}:{}:{}", hour, minute, second);
        let data: [u8; 4] = [minute / 10, minute % 10, second / 10, second % 10];

        ic_tm1637.i2c_bus_write_command(0x44);
        for i in 0..4 {
            ic_tm1637.i2c_bus_start();
            ic_tm1637.i2c_bus_write_byte(I2C_ADDR[i]);
            ic_tm1637.i2c_bus_write_byte(NUMS[data[i] as usize]);
            ic_tm1637.i2c_bus_start();
        }
        ic_tm1637.i2c_bus_write_command(0x8f);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}
