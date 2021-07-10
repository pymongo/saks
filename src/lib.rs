/*!
Swiss Army Knife Shield for Raspberry Pi
*/
// #![warn(clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
pub mod pin_map;

//type GpioReg = u32;
/// https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf page 65
const BCM2711_GPIO_REGISTER_COUNT: usize = 58;
const GPIO_MMAP_LEN: usize = BCM2711_GPIO_REGISTER_COUNT * std::mem::size_of::<u32>();
const GPFSEL0: usize = 0x00;
const GPSET0: usize = 0x1c / std::mem::size_of::<u32>();
const GPCLR0: usize = 0x28 / std::mem::size_of::<u32>();
// const GPLEV0: usize = 0x34 / std::mem::size_of::<u32>();

/// In BCM pin numbering
#[repr(u32)]
pub enum SaksPins {
    Buzzer = 12,
}

impl From<SaksPins> for u32 {
    fn from(pin_num: SaksPins) -> Self {
        pin_num as Self
    }
}

pub struct Gpio {
    mapped_addr: *mut u32,
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

impl Gpio {
    #[must_use]
    pub unsafe fn new() -> Self {
        let model = std::fs::read_to_string("/sys/firmware/devicetree/base/model").unwrap();
        if !model.starts_with("Raspberry Pi 4 Model") {
            panic!("only support Raspberry Pi 4");
        }

        let fd = libc::open("/dev/gpiomem\0".as_ptr().cast(), libc::O_RDWR);
        if fd == -1 {
            panic!("{}", std::io::Error::last_os_error());
        }
        // /dev/gpiomem doesn't need offset
        let mapped_addr = libc::mmap(
            std::ptr::null_mut(),
            GPIO_MMAP_LEN,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if mapped_addr == libc::MAP_FAILED {
            panic!("{}", std::io::Error::last_os_error());
        }
        libc::close(fd);
        Self {
            mapped_addr: mapped_addr.cast(),
        }
    }

    pub unsafe fn set_mode(&self, pin: SaksPins, pin_mode: PinMode) {
        let bcm_pin_num = u32::from(pin);
        // bcm2711-peripherals.pdf page 66, each GPFSEL${N} register every 3 bit manage a GPIO's pin_mode
        // eg. Buzzer=12 need register GPFSEL1 to set pin_mode, offset to mmap is 1
        let gpfsel_index = bcm_pin_num / 10;
        // eg. GPFSEL1 bit [6..=8] is the bit_mask to set Buzzer=12 pin_mode
        let pin_mode_shift = (bcm_pin_num % 10) * 3;

        let reg_ptr = self
            .mapped_addr
            .offset(GPFSEL0 as isize + gpfsel_index as isize);

        let mut reg_val = *reg_ptr;
        // eg. bit_mask to set GPFSEL1's [6..=8] bit to zero
        let bit_mask_to_clear_bcm_pin_num_pin_mode = !(0b111 << pin_mode_shift);
        reg_val &= bit_mask_to_clear_bcm_pin_num_pin_mode;
        // eg. set GPFSEL1's [6..=8] bit to pin_mode arg
        reg_val |= (u32::from(pin_mode)) << pin_mode_shift;

        *reg_ptr = reg_val;
    }

    /// level=true set pin to high voltage level, vice versa
    /// bcm2711-peripherals.pdf page 69
    pub unsafe fn set_level(&self, pin: SaksPins, level: bool) {
        let bcm_pin_num = u32::from(pin);
        // pin 32..=57 use GPSET1/GPCLR1/GPLEV1 register, saks shied all pin number is less than 32
        let addr_offset = if level { GPSET0 } else { GPCLR0 } as isize + bcm_pin_num as isize / 32;
        // eg. 只把bit12设置成1其余为0的话，只会对GPIO发出设置成高电平的命令，其他GPIO不会受到影响
        // 并不会像51单片机那样每个引脚的电平就是寄存器当前的值
        // 所以当BCM12的GPSET0设成1,GPCLR0也设置成1的话，看哪一个是最后设置的，最后设置的那个生效
        *self.mapped_addr.offset(addr_offset) = 1 << (bcm_pin_num % 32);
    }

    // /// return true if pin current is high voltage level, vice versa
    // pub unsafe fn get_level(&self, pin :SaksPins) -> bool {
    //     todo!()
    // }

    pub unsafe fn clear_all(&self) {
        *self.mapped_addr.add(GPCLR0) = 0;
        *self.mapped_addr.add(GPCLR0 + 1) = 0;
    }
}

impl Drop for Gpio {
    fn drop(&mut self) {
        unsafe {
            self.clear_all();
        }
    }
}

#[cfg(test)]
unsafe fn test_buzzer() {
    let gpio = Gpio::new();
    gpio.set_mode(SaksPins::Buzzer, PinMode::Input);
    for _ in 0..5 {
        gpio.set_level(SaksPins::Buzzer, true);
        libc::sleep(1);
        gpio.set_level(SaksPins::Buzzer, false);
        libc::sleep(0);
    }
}

#[test]
fn feature() {
    unsafe {
        test_buzzer();
    }
}
