use xkbcommon::xkb;

use crate::{ime::ImeState, keyboard::KbState, wayland::WaylandState};

pub struct State {
    pub wayland: WaylandState,

    pub kb: KbState,
    pub ime: ImeState,
}

impl State {
    pub fn new() -> Self {
        State {
            wayland: WaylandState::default(),
            kb: KbState {
                context: xkb::Context::new(xkb::CONTEXT_NO_FLAGS),
                keymap: None,
                state: None,
            },
            ime: ImeState::default(),
        }
    }
}
