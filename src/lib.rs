//! Swiss Army Knife Shield for Raspberry Pi 4
// #![warn(clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
/// https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf page 65
const BCM2711_GPIO_REGISTER_COUNT: usize = 58;
const GPIO_MMAP_LEN: usize = BCM2711_GPIO_REGISTER_COUNT * std::mem::size_of::<u32>();
const GPFSEL0: usize = 0x00;
const GPSET0: usize = 0x1c / std::mem::size_of::<u32>();
const GPCLR0: usize = 0x28 / std::mem::size_of::<u32>();
const GPLEV0: usize = 0x34 / std::mem::size_of::<u32>();

/// BCM pin numbering define in bcm2711-peripherals.pdf
/// [SAKS official website pin mapping page](https://shumeipai.nxez.com/swiss-army-knife-shield-for-raspberry-pi)
#[repr(u32)]
pub enum SaksPins {
    /// Active buzzer, beep if VoltageLevel::Low
    Buzzer = 12,

    /// LED row port with 74HC595
    Ds = 6,
    Stcp = 13,
    Shcp = 19,

    /// 4-digit seven segment display port with TM1637
    Di = 25,
    Clk = 5,
}

impl From<SaksPins> for u32 {
    fn from(pin_num: SaksPins) -> Self {
        pin_num as Self
    }
}

/**
bcm2711-peripherals.pdf page 66:

> The FSELn field determines the functionality of the nth GPIO pin

GPFSEL0-GPFSEL5: 这组6个寄存器都是控制每个GPIO的模式
每个寄存器每3bit控制一个GPIO，例如GPFSEL0的bit[3..=5]控制GPIO1的模式
bit[30..=31]没用，仅用于32bit对齐寄存器

例如蜂鸣器连向BCM12的引脚，计算12/10得知需要GPFSEL1来控制蜂鸣器引脚的模式
也就是mmap上索引为1的寄存器
*/
#[repr(u32)]
pub enum PinMode {
    Input = 0b000,
    Output = 0b001,
}

impl From<PinMode> for u32 {
    fn from(pin_mode: PinMode) -> Self {
        pin_mode as Self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum VoltageLevel {
    Low = 0,
    High = 1,
}

impl From<VoltageLevel> for bool {
    #[inline]
    fn from(level: VoltageLevel) -> Self {
        match level {
            VoltageLevel::Low => false,
            VoltageLevel::High => true,
        }
    }
}

impl std::ops::Not for VoltageLevel {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        match self {
            Self::Low => Self::High,
            Self::High => Self::Low,
        }
    }
}

pub struct Saks {
    mapped_addr: *mut u32,
}

impl Default for Saks {
    fn default() -> Self {
        Self::new()
    }
}

impl Saks {
    #[must_use]
    pub fn new() -> Self {
        let model = std::fs::read_to_string("/sys/firmware/devicetree/base/model").unwrap();
        if !model.starts_with("Raspberry Pi 4 Model") {
            panic!("only support Raspberry Pi 4");
        }

        let fd = unsafe { libc::open("/dev/gpiomem\0".as_ptr().cast(), libc::O_RDWR) };
        if fd == -1 {
            panic!("{}", std::io::Error::last_os_error());
        }
        let mapped_addr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                GPIO_MMAP_LEN,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0, // /dev/gpiomem doesn't need offset
            )
        };
        if mapped_addr == libc::MAP_FAILED {
            panic!("{}", std::io::Error::last_os_error());
        }
        unsafe {
            libc::close(fd);
        }
        let saks = Self {
            mapped_addr: mapped_addr.cast(),
        };
        saks.init_pins();
        saks
    }

    fn set_mode(&self, pin: SaksPins, pin_mode: PinMode) {
        let bcm_pin_num = u32::from(pin);
        // bcm2711-peripherals.pdf page 66, each GPFSEL${N} register every 3 bit manage a GPIO's pin_mode
        // eg. Buzzer=12 need register GPFSEL1 to set pin_mode, offset to mmap is 1
        let gpfsel_index = bcm_pin_num / 10;
        // eg. GPFSEL1 bit [6..=8] is the bit_mask to set Buzzer=12 pin_mode
        let pin_mode_shift = (bcm_pin_num % 10) * 3;

        let reg_ptr = unsafe { self.mapped_addr.add(GPFSEL0 + gpfsel_index as usize) };

        let mut reg_val = unsafe { *reg_ptr };
        // eg. bit_mask to set GPFSEL1's [6..=8] bit to zero
        let bit_mask_to_clear_bcm_pin_num_pin_mode = !(0b111 << pin_mode_shift);
        reg_val &= bit_mask_to_clear_bcm_pin_num_pin_mode;
        // eg. set GPFSEL1's [6..=8] bit to pin_mode arg
        reg_val |= (u32::from(pin_mode)) << pin_mode_shift;

        unsafe {
            *reg_ptr = reg_val;
        }
    }

    /// level=true set pin to high voltage level, vice versa
    /// bcm2711-peripherals.pdf page 69
    pub fn set_level(&self, pin: SaksPins, level: VoltageLevel) {
        self.set_is_high_level(pin, bool::from(level));
    }

    pub fn set_is_high_level(&self, pin: SaksPins, is_high_level: bool) {
        let bcm_pin_num = u32::from(pin);
        // pin 32..=57 use GPSET1/GPCLR1/GPLEV1 register, saks shied all pin number is less than 32
        let addr_offset = if is_high_level { GPSET0 } else { GPCLR0 } + bcm_pin_num as usize / 32;
        // eg. 只把bit12设置成1其余为0的话，只会对GPIO发出设置成高电平的命令，其他GPIO不会受到影响
        // 并不会像51单片机那样每个引脚的电平就是寄存器当前的值
        // 所以当BCM12的GPSET0设成1,GPCLR0也设置成1的话，看哪一个是最后设置的，最后设置的那个生效
        unsafe {
            *self.mapped_addr.add(addr_offset) = 1 << (bcm_pin_num % 32);
        }
    }

    /// return if pin is_high_level, return true if pin current is high voltage level, vice versa
    pub fn get_level(&self, pin: SaksPins) -> VoltageLevel {
        let bcm_pin_num = u32::from(pin);
        let addr_offset = GPLEV0 + bcm_pin_num as usize / 32;
        let reg_val = unsafe { *self.mapped_addr.add(addr_offset) };
        if reg_val & (1 << (bcm_pin_num % 32)) == 0 {
            VoltageLevel::Low
        } else {
            VoltageLevel::High
        }
    }

    pub fn led_row_write_a_byte(&self, byte: u8) {
        for bit_mask in 0..8 {
            self.set_is_high_level(SaksPins::Ds, (byte >> bit_mask) & 1 == 1);
            self.set_level(SaksPins::Shcp, VoltageLevel::Low);
            self.set_level(SaksPins::Shcp, VoltageLevel::High);
        }
        // 往STCP管脚写一个上升沿的脉冲信号
        self.set_level(SaksPins::Stcp, VoltageLevel::Low);
        self.set_level(SaksPins::Stcp, VoltageLevel::High);
    }

    fn init_pins(&self) {
        self.set_mode(SaksPins::Buzzer, PinMode::Output);
        // 74HC595
        self.set_mode(SaksPins::Ds, PinMode::Output);
        self.set_mode(SaksPins::Stcp, PinMode::Output);
        self.set_mode(SaksPins::Shcp, PinMode::Output);
        // TM1637 I2C
        self.set_mode(SaksPins::Di, PinMode::Output);
        self.set_mode(SaksPins::Clk, PinMode::Output);
    }

    unsafe fn clear_all(&self) {
        *self.mapped_addr.add(GPCLR0) = 0b1111_1111;
        *self.mapped_addr.add(GPCLR0 + 1) = 0b1111_1111;
    }

    pub fn i2c_bus_start(&self) {
        self.set_level(SaksPins::Clk, VoltageLevel::High);
        self.set_level(SaksPins::Di, VoltageLevel::High);
        self.i2c_bus_delay();
        self.set_level(SaksPins::Di, VoltageLevel::Low);
        self.i2c_bus_delay();
        self.set_level(SaksPins::Clk, VoltageLevel::Low);
        self.i2c_bus_delay();
    }

    pub fn i2c_bus_delay(&self) {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    pub fn i2c_bus_write_byte(&self, byte: u8) {
        for bit_mask in 0..8 {
            let di_level = (byte >> bit_mask) & 1 == 1;
            self.set_level(SaksPins::Clk, VoltageLevel::Low);
            self.i2c_bus_delay();
            if di_level {
                self.set_level(SaksPins::Di, VoltageLevel::High);
            } else {
                self.set_level(SaksPins::Di, VoltageLevel::Low);
            }
            self.i2c_bus_delay();
            self.set_level(SaksPins::Clk, VoltageLevel::High);
            self.i2c_bus_delay();
        }

        self.set_level(SaksPins::Clk, VoltageLevel::Low);
        self.i2c_bus_delay();
        self.set_level(SaksPins::Di, VoltageLevel::High);
        self.i2c_bus_delay();
        self.set_level(SaksPins::Clk, VoltageLevel::High);
        self.i2c_bus_delay();
    }

    pub fn i2c_bus_write_command(&self, command: u8) {
        self.i2c_bus_start();
        self.i2c_bus_write_byte(command);
        self.i2c_bus_start();
    }
}

impl Drop for Saks {
    fn drop(&mut self) {
        unsafe {
            self.clear_all();
            self.set_level(SaksPins::Buzzer, VoltageLevel::High);
            let ret = libc::munmap(self.mapped_addr.cast(), GPIO_MMAP_LEN);
            if ret == -1 {
                panic!("{}", std::io::Error::last_os_error());
            }
        }
    }
}
