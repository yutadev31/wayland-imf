use std::os::fd::OwnedFd;

use xkbcommon::xkb;

use crate::ime::ImeState;

pub struct KbState {
    pub context: xkb::Context,
    pub keymap: Option<xkb::Keymap>,
    pub state: Option<xkb::State>,
}

pub fn handle_keymap(fd: OwnedFd, size: u32, kb: &mut KbState) {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = unsafe {
        xkb::Keymap::new_from_fd(
            &context,
            fd,
            size.try_into().unwrap(),
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::COMPILE_NO_FLAGS,
        )
        .expect("Failed to create keymap")
    }
    .unwrap();

    let state = xkb::State::new(&keymap);

    kb.keymap = Some(keymap);
    kb.state = Some(state);
}

pub fn handle_modifiers(kb: &mut KbState, depressed: u32, latched: u32, locked: u32, group: u32) {
    if let Some(state) = &mut kb.state {
        state.update_mask(depressed, latched, locked, 0, 0, group);
    }
}

pub fn handle_key(kb: &mut KbState, key: u32, ime: &mut ImeState) -> bool {
    if let Some(state) = &kb.state {
        let keycode = xkb::Keycode::new(key + 8);

        let sym = state.key_get_one_sym(keycode);

        match sym {
            xkb::Keysym::space => {
                ime.space();
            }
            xkb::Keysym::BackSpace => {
                return ime.backspace();
            }
            xkb::Keysym::Return => {
                return ime.enter();
            }
            xkb::Keysym::Zenkaku_Hankaku => {
                ime.switch_mode();
            }
            _ => {
                let text = state.key_get_utf8(keycode);
                if !text.is_empty() && !text.chars().any(|c| c.is_control()) {
                    return ime.input_char(text);
                }
            }
        }
    }

    return false;
}
