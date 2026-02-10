# Wake up... The terminal has you. Follow the white rat.

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

And it's a new kind of problem.

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

# Dealve

Find the best game deals across Steam, Epic Games & more.

> github.com/kurama/dealve-tui

---

# Dealve

![image:center](dealve.png)

---

# Minesweeper 4D

4d Minesweeper TUI

> gh/itabesamesa/minesweeper_4d_rs

---

# A terminal renaissance?

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

# Ok, what's going on? Really...

<!-- background: hyper -->

---

# "ratatuify"

"To rebuild or redesign something using the Rust library Ratatui,
often transforming a GUI into an interactive terminal UI that's
more performant and way more fancy."

> "No wonder why your GUI runs so slow. Hold my cheese, let me Ratatuify it."

https://www.urbandictionary.com/define.php?term=ratatuify

---

# Rat in the Wild

Push Ratatui to the limit!  
Winners:

1. suzui-rs
2. ratatui-minecraft
3. texaform

e.g. control robots on a remote planet over TCP and track their progress in the terminal :o

> ratatui/ratatui/discussions/1886

---

# Embedded?

![image:left](rat-cheese.png)

> Mousefood!

Ratatui backend for embedded graphics

---

# From servers to toasters?

<!-- background: aurora -->

---

# Rat in Paris

![image:left](rat-in-paris.png)

panicked at library/alloc/src/alloc.rs:439:13:

cheese allocation of 113920 bytes failed

---

# TODO

Remember backend agnostic?
We can use any backend
Rat in the wild
Mousefood
This presentation tool
Tuitar
What's next? -> Rust powers the world
Some meaningful message (about hacking, rats, etc.?)

- World needs better care
  Hire me & sponsor me
  I'm out

---

# YouTube

![image:center](youtube.png)
