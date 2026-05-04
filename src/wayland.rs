use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2, zwp_input_method_v2,
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

            if interface == "wl_seat" {
                state.seat =
                    Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(1), qh, ()));
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
        }
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

impl Dispatch<zwp_input_method_v2::ZwpInputMethodV2, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &zwp_input_method_v2::ZwpInputMethodV2,
        event: <zwp_input_method_v2::ZwpInputMethodV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                println!("IME activated");
                state.keyboard_grab = Some(proxy.grab_keyboard(qh, ()));
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
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { fd, size, .. } => {
                handle_keymap(fd, size, &mut state.kb);
            }
            zwp_input_method_keyboard_grab_v2::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
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
                    wayland_client::protocol::wl_keyboard::KeyState::Pressed => {
                        println!("key pressed: {}", key);

                        handle_key(&mut state.kb, key, &mut state.ime);

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
