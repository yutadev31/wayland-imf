use wayland_client::protocol::wl_seat;
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2, zwp_input_method_v2,
};
use xkbcommon::xkb;

use crate::{ime::ImeState, keyboard::KbState};

pub struct State {
    pub seat: Option<wl_seat::WlSeat>,
    pub input_method: Option<zwp_input_method_v2::ZwpInputMethodV2>,
    pub im_manager: Option<zwp_input_method_manager_v2::ZwpInputMethodManagerV2>,
    pub keyboard_grab: Option<zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2>,

    pub kb: KbState,
    pub ime: ImeState,
}

impl State {
    pub fn new() -> Self {
        State {
            seat: None,
            input_method: None,
            im_manager: None,
            keyboard_grab: None,
            kb: KbState {
                context: xkb::Context::new(xkb::CONTEXT_NO_FLAGS),
                keymap: None,
                state: None,
            },
            ime: ImeState::default(),
        }
    }
}
