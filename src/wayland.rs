use std::os::fd::{AsRawFd, BorrowedFd};

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_compositor, wl_registry, wl_seat, wl_surface},
};
use wayland_protocols::xdg::shell::client::xdg_wm_base;
use wayland_protocols_misc::{
    zwp_input_method_v2::client::{
        zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2, zwp_input_method_v2,
    },
    zwp_virtual_keyboard_v1::client::{zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1},
};

use crate::{
    keyboard::{handle_key, handle_keymap, handle_modifiers},
    state::State,
};

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("global: {} v{}", interface, version);

            if interface == "wl_compositor" {
                state.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }

            if interface == "wl_seat" {
                state.seat =
                    Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(1), qh, ()));
            }

            if interface == "xdg_wm_base" {
                state.xdg_wm_base = Some(registry.bind::<xdg_wm_base::XdgWmBase, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }

            if interface == "zwp_input_method_manager_v2" {
                state.im_manager = Some(
                    registry.bind::<zwp_input_method_manager_v2::ZwpInputMethodManagerV2, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    ),
                );
            }

            if interface == "zwp_virtual_keyboard_manager_v1" {
                state.vk_manager = Some(
                    registry
                        .bind::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, _, _>(
                            name,
                            version.min(1),
                            qh,
                            (),
                        ),
                );
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &xdg_wm_base::XdgWmBase,
        _event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_input_method_v2::ZwpInputMethodV2, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &zwp_input_method_v2::ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                println!("IME activated");
                let keyboard_grab = proxy.grab_keyboard(qh, ());
                state.keyboard_grab = Some(keyboard_grab);
            }
            _ => {}
        }
    }
}

impl Dispatch<zwp_input_method_manager_v2::ZwpInputMethodManagerV2, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
        _event: zwp_input_method_manager_v2::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2,
        event: <zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { fd, size, .. } => {
                if let Some(vk) = &state.virtual_keyboard {
                    vk.keymap(1, unsafe { BorrowedFd::borrow_raw(fd.as_raw_fd()) }, size);
                }

                handle_keymap(fd, size, &mut state.kb);
            }
            zwp_input_method_keyboard_grab_v2::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                if let Some(vk) = &state.virtual_keyboard {
                    vk.modifiers(mods_depressed, mods_latched, mods_locked, group);
                }

                handle_modifiers(
                    &mut state.kb,
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    group,
                );
            }
            zwp_input_method_keyboard_grab_v2::Event::Key {
                key,
                state: key_state,
                ..
            } => match key_state {
                wayland_client::WEnum::Value(key_state) => match key_state {
                    wayland_client::protocol::wl_keyboard::KeyState::Released => {
                        if let Some(vk) = &state.virtual_keyboard {
                            vk.key(16, key, 0);
                        }
                    }
                    wayland_client::protocol::wl_keyboard::KeyState::Pressed => {
                        println!("key pressed: {}", key);

                        handle_key(&mut state.kb, key, &mut state.ime);

                        if !state.ime.ime_enabled {
                            if let Some(vk) = &state.virtual_keyboard {
                                vk.key(16, key, 1);
                            }
                        }

                        if let Some(im) = &state.input_method {
                            if !state.ime.commit_buf.is_empty() {
                                let buf = state.ime.commit_buf.clone();
                                im.commit_string(buf);
                                state.ime.commit_buf.clear();
                            }

                            if state.ime.commit_pending {
                                let buf = state.ime.preedit_kana.clone();
                                im.commit_string(buf);
                                state.ime.preedit_raw.clear();
                                state.ime.preedit_kana.clear();
                                state.ime.commit_pending = false;
                            }

                            im.set_preedit_string(
                                state.ime.preedit_kana.clone(),
                                state.ime.preedit_kana.len().try_into().unwrap(),
                                state.ime.preedit_kana.len().try_into().unwrap(),
                            );

                            im.commit(0);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
}

impl Dispatch<zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
        _event: <zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
        _event: <zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
