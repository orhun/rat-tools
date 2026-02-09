# Wake up... The terminal has you. Follow the white rat.

<!-- background: nebula -->

---

# The rat in the room

This entire talk is rendered on a 240×320 display.

Using `Rust` & `Ratatui`!

- RP2040-Zero microcontroller
- ST7789 display
- Hella cheese!

Let's talk about this.

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

> https://www.tbench.ai

---

# tbench

![image:center](terminal-bench.png)

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

# https://www.reddit.com/r/commandline/comments/1qyq204/why_do_so_many_tui_projects_seem_to_use_rust_as/

![image:center](r-commandline-2.png)

---

# Ratatui

![image:center](ratatui-header.png)

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

# Minesweeper 4D

![image:center](minesweeper-4d.png)

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

# suzui-rs

Running Ratatui on a Suzuki Baleno.

> github.com/thatdevsherry/suzui-rs

Suzuki Serial Data Line (SDL) viewer in Rust.

---

# suzui-rs

![image:center](suzui-rs.png)

---

# From servers to toasters?

<!-- background: aurora -->
