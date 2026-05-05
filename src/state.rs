use std::collections::HashMap;

use wayland_client::protocol::{wl_compositor, wl_seat};
use wayland_protocols::xdg::shell::client::xdg_wm_base;
use wayland_protocols_misc::{
    zwp_input_method_v2::client::{
        zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2, zwp_input_method_v2,
    },
    zwp_virtual_keyboard_v1::client::{
        zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    },
};
use xkbcommon::xkb;

use crate::{dict::load_dict, ime::ImeState, keyboard::KbState};

pub struct State {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub seat: Option<wl_seat::WlSeat>,
    pub xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub input_method: Option<zwp_input_method_v2::ZwpInputMethodV2>,
    pub im_manager: Option<zwp_input_method_manager_v2::ZwpInputMethodManagerV2>,
    pub keyboard_grab: Option<zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2>,
    pub virtual_keyboard: Option<ZwpVirtualKeyboardV1>,
    pub vk_manager: Option<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1>,

    pub kb: KbState,
    pub ime: ImeState,
    pub dict: HashMap<String, Vec<String>>,
}

impl State {
    pub fn new() -> Self {
        State {
            compositor: None,
            seat: None,
            xdg_wm_base: None,
            input_method: None,
            im_manager: None,
            keyboard_grab: None,
            virtual_keyboard: None,
            vk_manager: None,
            kb: KbState {
                context: xkb::Context::new(xkb::CONTEXT_NO_FLAGS),
                keymap: None,
                state: None,
            },
            ime: ImeState::default(),
            dict: load_dict(),
        }
    }
}
