# WAKE UP. THE TERMINAL HAS YOU. FOLLOW THE WHITE RAT.

<!-- background: nebula -->

---

# The rat in the room

This entire talk is rendered on a 240×320 display.

Using `Rust` & `Ratatui`!

1. RP2040-Zero microcontroller
2. ST7789 display
3. Hella cheese!

> Let's talk about this.

---

# <intro1>

---

# <intro2>

---

# init 0

![image:left](vt100.png)

Before modern computing, we had simple **terminals**.

And we learned to do a lot with very little.

---

# Then...

![image:left](gemini.png)

They changed.

Like everything else.

They evolved.

### Because we always wanted more.

---

# Why terminals in 2026?

They are **timeless**, and still relevant.

Building high quality and modern terminal applications is still a problem.

> And it's a new kind of problem.

---

# Terminal Bench

Agent performance benchmarks.

1. GPT-5.3-Codex (75.1%)
2. Claude Opus 4.6 (69.9%)
3. GPT-5.2 (64.9%)
4. Gemini 3 Pro (64.7%)

> https://www.tbench.ai

---

# codex

![image:center](codex-notice.png)

---

# Ecosystem

![image:left](rat-cup.png)

It isn't just about Codex.

> There is a pattern.

A lot of modern terminal software
is built with Rust and Ratatui.

---

# https://www.reddit.com/r/commandline/comments/1qyq204/why_do_so_many_tui_projects_seem_to_use_rust_as/

![image:center](r-commandline.png)

---

# u/HeyCanIBorrowThat

> "I think ratatui makes it really easy to make TUI apps compared to older frameworks like ncurses.
> Rust is also trendy right now and for good reason"

---

# <mascot>

---

# Ratatui

> A Rust library for cooking up TUIs

250+ contributors, hundreds of apps, 15M+ crate downloads

`gitui`, `atuin`, `yazi`, `skim`, `dioxus-cli`, `tokio-console` & more!

Used by Netflix, AWS, Oxide & more!

> https://ratatui.rs

---

# Vortix

Terminal UI for WireGuard and OpenVPN with real-time telemetry and leak guarding.

> github.com/Harry-kp/vortix

---

# Vortix

![image:center](vortix.png)

---

# Crabsid

A TUI music player for Commodore 64 SID tunes

> github.com/mlund/crabsid

---

# Crabsid

![image:center](crabsid.png)

---

# eprofiler-tui

eBPF profiler flamegraph based TUI

> github.com/rogercoll/eprofiler-tui

---

# eprofiler-tui

![image:center](eprofiler.png)

---

# A TERMINAL RENAISSANCE?

<!-- This rat library keeps popping up everywhere! -->

<!-- background: waves -->

---

# Why?

![image:left](rat-ski.png)

1. Performance
2. Safety
3. Ergonomics
4. Ecosystem\*
5. Portability\*

> Rust makes this possible!

---

# The secret sauce

1. Immediate mode rendering
2. Declarative UI
3. Backend-agnostic events & rendering
   - e.g. crossterm
   - e.g. embedded-graphics

> "Given this state and these events, the UI renders as _this_, immediately."

### Events -> State -> UI -> Repeat

---

# Starting a Ratatui app

```rust
ratatui::run(|terminal| {
    // main loop
    loop {

        // render the UI
        terminal.draw(|frame| {
            // draw widgets
        })?;

        // handle events
    }
});
```

---

# Rendering

```rust
terminal.draw(|frame| {
    let area = frame.area();

    let block = Block::bordered()
        .title("Rendering");

    frame.render_widget(
        block,
        area
    );
})?;
```

---

# <demo-table-scrollbar>

---

# <demo-sparkline>

---

# <demo-linegauge>

---

# <demo-gauge>

---

# <demo-chart>

---

# <demo-canvas>

---

# <demo-barchart>

---

# Custom widgets

Widgets are just `Widget::render`:

```rust
struct CheeseMeter { value: u16 }

impl Widget for CheeseMeter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw cells directly
    }
}
```

---

# <custom-widget>

---

# Event handling

```rust
struct App {
    counter: usize,
}

let event = backend.poll()?;
match event {
    Event::Key(Key::Char('q'))
        => break,
    Event::Key(Key::Up)
        => app.counter += 1,
}
```

---

# Wanna learn more?

1. https://ratatui.rs
2. github.com/ratatui/templates

```sh
$ cargo install cargo-generate

$ cargo generate ratatui/templates
```

3. github.com/ratatui/awesome-ratatui

---

# More!

![image:left](rat-demand.png)

1. Tools.
2. Games.
3. Dashboards.
4. ?????????

This is not even our final form.

> So much cheese to eat.

---

# Remember, backend-agnostic!

```rust
// explicit backend setup
let backend = CrosstermBackend::new(stdout());

let terminal = Terminal::new(backend)?;

terminal.draw(|frame| {
    // draw widgets
})?;
```

---

# Backend trait

```rust
pub trait Backend {
    fn draw<(&mut self, cells: I);
    fn clear(&mut self);
    fn flush(&mut self);
    fn size(&self);
    fn get_cursor(&self);

    // cursor, scrolling,
    // window size, etc.
}
```

---

# More backends!

0. **crossterm** for terminal apps
1. **mousefood** for embedded-graphics
2. **ratatui-wgpu** for GPU-accelerated rendering
3. **egui_ratatui** for EGUI widgets
4. **soft_ratatui** for arbitrary buffers
5. **bevy_ratatui** for Bevy apps
6. **ratzilla** for websites

> The possibilities are endless!

---

# <let-him-cook>

![image:left](lethimcook.png)

---

# suzui-rs

![image:center](suzui-rs.png)

---

# suzui-rs

Running Ratatui on a Suzuki Baleno.

> github.com/thatdevsherry/suzui-rs

Suzuki Serial Data Line (SDL) viewer in Rust.

---

# ratatui-psp

![image:center](ratatui-psp.png)

---

# ratatui-psp

Ratatui on the PlayStation Portable (PSP)!

> gh/overdrivenpotato/rust-psp

---

# tui-uefi

Build TUIs for UEFI firmware interfaces, like your BIOS!

> github.com/reubeno/tui-uefi

---

# OK, WHAT'S GOING ON? REALLY...

<!-- background: hyper -->

---

# "ratatuify"

"To rebuild or redesign something using the Rust library Ratatui,
often transforming a GUI into an interactive terminal UI that's
more performant and way more fancy."

> "No wonder why your GUI runs so slow. Hold my cheese, let me Ratatuify it."

https://www.urbandictionary.com/define.php?term=ratatuify

---

# Embedded?

![image:left](rat-cheese.png)

> Mousefood!

Ratatui backend for embedded graphics

Supports ESP32, RP2040, STM32,
e-ink displays & more!

---

# How?

1. Ratatui widgets produce cells (glyph + fg/bg + style)
2. Mousefood maps cells to pixels via font + color theme
3. The display driver receives drawn pixels

> github.com/ratatui/mousefood

---

# Backend setup

```rust
// Pick your display driver
let display = Ssd1306::new(interface, Size72x40, Rotate0);

let backend = EmbeddedBackend::new(&mut display, config);

let terminal = Terminal::new(backend)?;
```

---

# Render as usual

```rust
loop {
    terminal.draw(|frame| {
      // render widgets here
    })?;
}
```

Events -> State -> UI -> Pixels
-> Flush

---

# Font support

Ratatui relies on box-drawing, braille, and other Unicode glyphs.

Many built-in embedded-graphics fonts only cover ASCII/ISO/JIS

Mousefood ships Unicode fonts so most widgets render correctly!

⠁⠃⠉⠙⠑⠋⠓⠟⠿⣿  
┌─┬─┐ ┏━┳━┓ ╔═╦═╗ ░▒▓█  
│ │ │ ┃ ┃ ┃ ║ ║ ║ ▄▀▄▀▄

---

# Backend configuration

```rust
EmbeddedBackendConfig {
    font_regular: MONO_6X13,
    font_bold: Some(MONO_6X13_BOLD),
    font_italic: Some(MONO_6X13_ITALIC),
    ..Default::default()
};
```

---

# FROM SERVERS TO TOASTERS???? WHAT IS THIS PRESENTATION?

<!-- background: aurora -->

---

# <ratdeck-title>

<!-- background: waves -->

---

# Ratdeck

> Ratatui-powered slide deck.

- RP2040 + ST7789 @ 320x240
- Build-time slide generation
- All slides are Markdown
- Bundled images/assets
- Desktop simulator (SDL)

CPU: Dual-core Cortex-M0+ @ 133MHz

RAM: 256KB (heap ~200KB)

---

# How it works

1. Author slides in `slides.md`
2. Build script compiles slides + assets
3. RP2040 renders with Ratatui + Mousefood

> build.rs does the heavy lifting!

- Parse slides
- Extract and convert images
- Embed everything in the firmware

---

# Slide format

```md
# Title

![image:left](rat.png)

Markdown text here...
```

Custom backgrounds are supported!  
Rendered with Ratatui as usual.

---

# WE ARE JUST RATS IN A SIMULATION.

<!-- background: waves -->

---

# Images?

```rust
let image = resolve_image("rat.png");
let im = Image::new(image, Point::new(0, 0));

let display = terminal
  .backend_mut().display_mut();
im.draw(display);
```

---

# Limitations

- Raw RGB565 is 2 bytes/pixel
- 320x240 image ≈ 153,600 bytes
- Heap is ~200KB on device
- Too many assets = larger firmware

Maybe stream images from external SPI flash? (e.g. W25Qxx)

Or compression?

### No thanks, I'll just render this with my last 150kB:

---

# Rat in Paris

![image:left](rat-in-paris.png)

panicked at library/alloc/src/alloc.rs:439:13:

cheese allocation of 113920 bytes failed

---

# Our ecosystem

0. Ratatui
1. Mousefood
2. tui-big-text: large text
3. tui-markdown: markdown rendering
4. tachyonfx: animations

v \_\_\_\_ v  
 \(. , .)/ thanks Rust!  
//———\\

---

# What else?

![image:left](rat-demand.png)

1. Cheese locator
2. Antui
3. Tuitar

> gh/orhun/
> rat-tools

> gh/orhun/tuitar

---

# What's next?

Ratatui brings a new approach to portable UI development.

Terminals + embedded, same tools, same cheese.

Still a lot to build.

> Rust powers the world!

---

# Everything is livestreamed!

> https://youtube.com/@orhundev

```
    _, .---.__c--.
   (__( )_._( )_`_>
       `~~'  `~'
```

---

# YouTube

![image:center](youtube.png)

---

# <qr-youtube>

---

# <sponsor-me>

---

# <qr-github>

---

# <questions>

---
