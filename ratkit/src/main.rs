#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::cell::RefCell;
use cortex_m::prelude::_embedded_hal_timer_CountDown;
// Linked-List First Fit Heap allocator (feature = "llff")
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::i2c as eh1;
use embedded_hal_0_2::blocking::delay::DelayMs;
use embedded_hal_0_2::blocking::i2c as eh0;
use embedded_hal_bus::i2c::RefCellDevice;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use mpu6050::Mpu6050;
use ratatui::Terminal;
use rp2040_panic_usb_boot as _;

use fugit::ExtU32;
use fugit::RateExtU32;
use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
    entry,
    gpio::{FunctionI2C, Pins},
    pac,
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
    Timer, I2C,
};
use ssd1306::size::DisplaySize128x64;
use ssd1306::{mode::DisplayConfig, prelude::DisplayRotation, Ssd1306};
use ssd1315::interface::I2cDisplayInterface;
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::StringDescriptors,
    prelude::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

mod app;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

struct Eh1I2cCompat<I> {
    inner: I,
}

impl<I> Eh1I2cCompat<I> {
    fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> eh0::Write<eh0::SevenBitAddress> for Eh1I2cCompat<I>
where
    I: eh1::I2c<eh1::SevenBitAddress>,
{
    type Error = I::Error;

    fn write(&mut self, address: eh0::SevenBitAddress, bytes: &[u8]) -> Result<(), Self::Error> {
        self.inner.write(address, bytes)
    }
}

impl<I> eh0::Read<eh0::SevenBitAddress> for Eh1I2cCompat<I>
where
    I: eh1::I2c<eh1::SevenBitAddress>,
{
    type Error = I::Error;

    fn read(
        &mut self,
        address: eh0::SevenBitAddress,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.inner.read(address, buffer)
    }
}

impl<I> eh0::WriteRead<eh0::SevenBitAddress> for Eh1I2cCompat<I>
where
    I: eh1::I2c<eh1::SevenBitAddress>,
{
    type Error = I::Error;

    fn write_read(
        &mut self,
        address: eh0::SevenBitAddress,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.inner.write_read(address, bytes, buffer)
    }
}

struct SpinDelay<'a> {
    timer: &'a Timer,
}

impl<'a> SpinDelay<'a> {
    fn new(timer: &'a Timer) -> Self {
        Self { timer }
    }

    fn delay_us_internal(&self, us: u32) {
        let start = self.timer.get_counter().ticks();
        let target = start.saturating_add(us as u64);
        while self.timer.get_counter().ticks() < target {}
    }
}

impl DelayMs<u8> for SpinDelay<'_> {
    fn delay_ms(&mut self, ms: u8) {
        for _ in 0..ms {
            self.delay_us_internal(1000);
        }
    }
}

fn usb_log<B: usb_device::class_prelude::UsbBus>(serial: &mut SerialPort<'_, B>, msg: &str) {
    let _ = serial.write(msg.as_bytes());
    let _ = serial.write(b"\r\n");
}

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        // We need a pretty big heap for ratatui. if the device reconnects as UF2, you probably hit this limit
        const HEAP_SIZE: usize = 100000;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = timer.count_down();

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Rat company")
            .product("Serial port")
            .serial_number("TEST")])
        .unwrap()
        .device_class(USB_CLASS_CDC)
        .build();
    loop {
        usb_dev.poll(&mut [&mut serial]);

        if usb_dev.state() == UsbDeviceState::Configured {
            usb_log(&mut serial, "usb configured");
            break;
        }
    }

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut button = pins.gpio14.into_pull_up_input();
    let mut menu_button = pins.gpio15.into_pull_up_input();
    let mut buzzer = pins.gpio8.into_push_pull_output();
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin = AdcPin::new(pins.gpio26.into_floating_input()).unwrap();

    usb_log(&mut serial, "before i2c");
    let sda = pins
        .gpio6
        .into_pull_up_input()
        .into_function::<FunctionI2C>();
    let scl = pins
        .gpio7
        .into_pull_up_input()
        .into_function::<FunctionI2C>();

    let i2c = I2C::i2c1(
        pac.I2C1,
        sda,
        scl,
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );
    usb_log(&mut serial, "after i2c");

    let i2c = Box::leak(Box::new(RefCell::new(i2c)));
    let i2c_display = RefCellDevice::new(i2c);
    let i2c_mpu = Eh1I2cCompat::new(RefCellDevice::new(i2c));

    usb_log(&mut serial, "before display");
    let interface = I2cDisplayInterface::new_interface(i2c_display);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    if usb_dev.poll(&mut [&mut serial]) {
        usb_log(&mut serial, "display init");
    }

    usb_log(&mut serial, "after display");

    let config = EmbeddedBackendConfig {
        flush_callback: Box::new(move |d: &mut Ssd1306<_, _, _>| {
            d.flush().unwrap();
        }),
        ..Default::default()
    };

    let backend = EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = app::App::new();
    let mut mpu = Mpu6050::new(i2c_mpu);
    let mut imu_delay = SpinDelay::new(&timer);
    let _ = mpu.init(&mut imu_delay);

    loop {
        let now_ms = timer.get_counter().ticks() / 1000;
        let button_pressed = button.is_low().unwrap_or(false);
        let menu_pressed = menu_button.is_low().unwrap_or(false);
        let accel = mpu.get_acc().ok().map(|a| (a.x, a.y, a.z));
        app.tick(now_ms, button_pressed, menu_pressed, accel);

        if app.buzzer_on() {
            let _ = buzzer.set_high();
        } else {
            let _ = buzzer.set_low();
        }

        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();

        usb_log(&mut serial, "loop");

        delay.start(50.millis());
        let _ = nb::block!(delay.wait());
    }
}
