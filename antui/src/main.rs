#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use cortex_m::prelude::_embedded_hal_timer_CountDown;
// Linked-List First Fit Heap allocator (feature = "llff")
use embedded_alloc::LlffHeap as Heap;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
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
use ssd1306::{mode::DisplayConfig, prelude::DisplayRotation, size::DisplaySize72x40, Ssd1306};
use ssd1315::interface::I2cDisplayInterface;
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::StringDescriptors,
    prelude::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

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

    usb_log(&mut serial, "before i2c");
    let sda = pins
        .gpio0
        .into_pull_up_input()
        .into_function::<FunctionI2C>();
    let scl = pins
        .gpio1
        .into_pull_up_input()
        .into_function::<FunctionI2C>();

    let i2c = I2C::i2c0(
        pac.I2C0,
        sda,
        scl,
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );
    usb_log(&mut serial, "after i2c");

    usb_log(&mut serial, "before display");
    let interface = I2cDisplayInterface::new_interface(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
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
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => {
            usb_log(&mut serial, "terminal new error");
            loop {
                delay.start(50.millis());
                let _ = nb::block!(delay.wait());
            }
        }
    };

    loop {
        if let Err(_) = terminal.draw(|f| {
            let area = f.area();
            let block = ratatui::widgets::Block::bordered().title("rat");
            f.render_widget(block, area);
        }) {
            usb_log(&mut serial, "terminal draw error");
        }

        usb_log(&mut serial, "loop");

        delay.start(50.millis());
        let _ = nb::block!(delay.wait());
    }
}
