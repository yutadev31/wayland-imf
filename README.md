# Wayland IMF

A input method framework written in Rust.

## Keyboard layout

`KeyboardInputMethod` can use an explicit XKB layout instead of the compositor-provided one.

```bash
wayland-imf --layout us
```

You can also set `WAYLAND_IMF_KEYBOARD_LAYOUT=us`.
