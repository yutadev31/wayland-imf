use std::os::fd::{AsRawFd, BorrowedFd};

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_compositor, wl_registry, wl_seat, wl_shm, wl_surface},
};
use wayland_protocols::xdg::shell::client::xdg_wm_base;
use wayland_protocols_misc::{
    zwp_input_method_v2::client::{
        zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2, zwp_input_method_v2,
    },
    zwp_virtual_keyboard_v1::client::{
        zwp_virtual_keyboard_manager_v1,
        zwp_virtual_keyboard_v1::{self, ZwpVirtualKeyboardV1},
    },
};

use crate::{
    keyboard::{handle_key, handle_keymap, handle_modifiers},
    state::State,
};

#[derive(Debug, Default)]
pub struct WaylandState {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub seat: Option<wl_seat::WlSeat>,
    pub shm: Option<wl_shm::WlShm>,
    pub xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub input_method: Option<zwp_input_method_v2::ZwpInputMethodV2>,
    pub im_manager: Option<zwp_input_method_manager_v2::ZwpInputMethodManagerV2>,
    pub keyboard_grab: Option<zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2>,
    pub virtual_keyboard: Option<ZwpVirtualKeyboardV1>,
    pub vk_manager: Option<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1>,
}

fn bind_global(
    state: &mut State,
    registry: &wl_registry::WlRegistry,
    name: u32,
    interface: &str,
    version: u32,
    qh: &QueueHandle<State>,
) {
    match interface {
        "wl_compositor" => {
            state.wayland.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                name,
                version.min(1),
                qh,
                (),
            ));
        }
        "wl_seat" => {
            state.wayland.seat =
                Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(1), qh, ()));
        }
        "wl_shm" => {
            state.wayland.shm =
                Some(registry.bind::<wl_shm::WlShm, _, _>(name, version.min(1), qh, ()));
        }
        "xdg_wm_base" => {
            state.wayland.xdg_wm_base =
                Some(registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, version.min(1), qh, ()));
        }
        "zwp_input_method_manager_v2" => {
            state.wayland.im_manager = Some(
                registry.bind::<zwp_input_method_manager_v2::ZwpInputMethodManagerV2, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ),
            );
        }
        "zwp_virtual_keyboard_manager_v1" => {
            state.wayland.vk_manager = Some(
                registry
                    .bind::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    ),
            );
        }
        _ => {}
    }
}

fn handle_input_method_event(
    state: &mut State,
    proxy: &zwp_input_method_v2::ZwpInputMethodV2,
    event: zwp_input_method_v2::Event,
    qh: &QueueHandle<State>,
) {
    match event {
        zwp_input_method_v2::Event::Activate => {
            println!("IME activated");
            let keyboard_grab = proxy.grab_keyboard(qh, ());
            state.wayland.keyboard_grab = Some(keyboard_grab);
            state.refresh_candidate_popup();
        }
        zwp_input_method_v2::Event::Deactivate => {
            state.hide_candidate_popup();
        }
        _ => {}
    }
}

fn handle_key_released(state: &mut State, key: u32) {
    if let Some(vk) = &state.wayland.virtual_keyboard {
        vk.key(0, key, 0);
    }
}

fn handle_key_pressed(state: &mut State, key: u32) {
    println!("key pressed: {}", key);

    if !handle_key(&mut state.kb, key, &mut state.ime)
        && let Some(vk) = &state.wayland.virtual_keyboard
    {
        vk.key(0, key, 1);
    }

    state.ime.post_update_preedit();
    state.refresh_candidate_popup();
    sync_input_method(state);
}

fn sync_input_method(state: &mut State) {
    let Some(im) = state.wayland.input_method.as_ref() else {
        return;
    };

    if !state.ime.context.commit_buf.is_empty() {
        let buf = state.ime.context.commit_buf.clone();
        im.commit_string(buf);
        state.ime.context.commit_buf.clear();
    }

    let preedit = state.ime.get_preedit();
    let cursor = preedit.len().try_into().unwrap();
    im.set_preedit_string(preedit, cursor, cursor);
    im.commit(0);
}

fn handle_keyboard_grab_event(
    state: &mut State,
    event: <zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2 as wayland_client::Proxy>::Event,
) {
    match event {
        zwp_input_method_keyboard_grab_v2::Event::Keymap { fd, size, .. } => {
            if let Some(vk) = &state.wayland.virtual_keyboard {
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
            if let Some(vk) = &state.wayland.virtual_keyboard {
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
            wayland_client::WEnum::Value(
                wayland_client::protocol::wl_keyboard::KeyState::Released,
            ) => {
                handle_key_released(state, key);
            }
            wayland_client::WEnum::Value(
                wayland_client::protocol::wl_keyboard::KeyState::Pressed,
            ) => {
                handle_key_pressed(state, key);
            }
            _ => {}
        },
        _ => {}
    }
}

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
            bind_global(state, registry, name, &interface, version, qh);
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
        handle_input_method_event(state, proxy, event, qh);
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
        handle_keyboard_grab_event(state, event);
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
