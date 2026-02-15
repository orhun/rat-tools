#![no_std]
#![no_main]

extern crate alloc;

// Linked-List First Fit Heap allocator (feature = "llff")
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::MODE_0;
use mipidsi::options::Rotation;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use core::num::NonZeroUsize;
use ratatui::layout::Layout;
use ratatui::Terminal;
use rp2040_panic_usb_boot as _;

use embedded_hal_bus::spi::ExclusiveDevice;
use fugit::RateExtU32;
use mipidsi::models::ST7789;
use mipidsi::Builder;
use mipidsi::{interface::SpiInterface, options::Orientation};
use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
    entry,
    gpio::{FunctionSpi, Pins},
    pac,
    sio::Sio,
    spi::Spi,
    usb::UsbBus,
    watchdog::Watchdog,
    Timer,
};
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::StringDescriptors,
    prelude::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use ratdeck::app::App;
use ratdeck::font_8x13::mono_8x13_atlas;
use ratdeck::font_8x13B::mono_8x13_bold_atlas;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const DI_BUFFER_LEN: usize = 512;
static mut DI_BUFFER: [u8; DI_BUFFER_LEN] = [0u8; DI_BUFFER_LEN];

fn usb_log<B: usb_device::class_prelude::UsbBus>(serial: &mut SerialPort<'_, B>, msg: &str) {
    let _ = serial.write(msg.as_bytes());
    let _ = serial.write(b"\r\n");
}

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        // We need a pretty big heap for ratatui. if the device reconnects as UF2, you probably hit this limit
        const HEAP_SIZE: usize = 200_000;
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

    let mut delay = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

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
    let mut button_next = pins.gpio2.into_pull_up_input();
    let mut button_prev = pins.gpio3.into_pull_up_input();

    usb_log(&mut serial, "before spi");

    let sck = pins.gpio10.into_function::<FunctionSpi>();
    let mosi = pins.gpio11.into_function::<FunctionSpi>();
    let cs = pins.gpio9.into_push_pull_output();
    let dc = pins.gpio12.into_push_pull_output();
    let rst = pins.gpio13.into_push_pull_output();
    let mut bl = pins.gpio14.into_push_pull_output();
    bl.set_high().unwrap();

    let spi = Spi::<_, _, _, 8>::new(pac.SPI1, (mosi, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        32_000_000u32.Hz(),
        MODE_0,
    );
    let spi = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    usb_log(&mut serial, "after spi");

    usb_log(&mut serial, "before display");

    let di = unsafe {
        let buf_ptr = core::ptr::addr_of_mut!(DI_BUFFER) as *mut u8;
        let buf = core::slice::from_raw_parts_mut(buf_ptr, DI_BUFFER_LEN);
        SpiInterface::new(spi, dc, buf)
    };

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    if usb_dev.poll(&mut [&mut serial]) {
        usb_log(&mut serial, "display init");
    }

    usb_log(&mut serial, "after display");

    Layout::init_cache(NonZeroUsize::new(20).unwrap());

    let config = EmbeddedBackendConfig {
        font_regular: mono_8x13_atlas(),
        font_bold: Some(mono_8x13_bold_atlas()),
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new();
    let mut last_frame = delay.get_counter();

    loop {
        let now = delay.get_counter();
        let elapsed_ms = (now - last_frame).to_millis() as u32;
        last_frame = now;

        terminal
            .draw(|f| {
                app.render(f, elapsed_ms);
            })
            .unwrap();

        app.render_image(terminal.backend_mut().display_mut());

        if button_next.is_low().unwrap_or(false) {
            app.next_slide();
            delay.delay_ms(120);
        } else if button_prev.is_low().unwrap_or(false) {
            app.prev_slide();
            delay.delay_ms(120);
        }
    }
}
