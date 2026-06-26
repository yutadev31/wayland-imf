use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::fd::{AsFd, OwnedFd},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use imf_core::KeyAction;
use xkbcommon::xkb;

use crate::ime::ImeEngine;

#[derive(Debug, Clone, Default)]
pub struct KeyboardConfig {
    pub layout: Option<String>,
}

pub struct KbState {
    pub context: xkb::Context,
    pub keymap: Option<xkb::Keymap>,
    pub state: Option<xkb::State>,
    pub config: KeyboardConfig,
}

fn configured_keymap(kb: &KbState) -> Result<Option<xkb::Keymap>, String> {
    let Some(layout) = kb.config.layout.as_deref() else {
        return Ok(None);
    };

    xkb::Keymap::new_from_names(&kb.context, "", "", layout, "", None, xkb::COMPILE_NO_FLAGS)
        .ok_or_else(|| format!("failed to compile XKB keymap for layout `{layout}`"))
        .map(Some)
}

fn apply_keymap(kb: &mut KbState, keymap: xkb::Keymap) {
    let state = xkb::State::new(&keymap);
    kb.keymap = Some(keymap);
    kb.state = Some(state);
}

fn create_temp_keymap_file(contents: &[u8]) -> std::io::Result<(File, u32)> {
    let size = contents.len().try_into().unwrap();
    let base = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    for attempt in 0..16 {
        let path: PathBuf = base.join(format!(
            "wayland-imf-keymap-{}-{nanos}-{attempt}.xkb",
            std::process::id()
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        };
        file.write_all(contents)?;
        file.sync_all()?;
        let _ = std::fs::remove_file(&path);
        return Ok((file, size));
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "failed to allocate temporary keymap file",
    ))
}

pub fn send_virtual_keyboard_keymap(
    kb: &KbState,
    vk: &wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    fd: &impl AsFd,
    size: u32,
) {
    match configured_keymap(kb) {
        Ok(Some(keymap)) => {
            let mut keymap_text = keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1);
            keymap_text.push('\0');
            match create_temp_keymap_file(keymap_text.as_bytes()) {
                Ok((file, size)) => vk.keymap(1, file.as_fd(), size),
                Err(err) => {
                    eprintln!("Failed to prepare override keymap file: {err}");
                    vk.keymap(1, fd.as_fd(), size);
                }
            }
        }
        Ok(None) => vk.keymap(1, fd.as_fd(), size),
        Err(err) => {
            eprintln!("{err}");
            vk.keymap(1, fd.as_fd(), size);
        }
    }
}

pub fn handle_keymap(fd: OwnedFd, size: u32, kb: &mut KbState) {
    match configured_keymap(kb) {
        Ok(Some(keymap)) => apply_keymap(kb, keymap),
        Ok(None) => {
            let keymap = unsafe {
                xkb::Keymap::new_from_fd(
                    &kb.context,
                    fd,
                    size.try_into().unwrap(),
                    xkb::KEYMAP_FORMAT_TEXT_V1,
                    xkb::COMPILE_NO_FLAGS,
                )
                .expect("Failed to create keymap")
            }
            .unwrap();

            apply_keymap(kb, keymap);
        }
        Err(err) => {
            eprintln!("{err}");
            let keymap = unsafe {
                xkb::Keymap::new_from_fd(
                    &kb.context,
                    fd,
                    size.try_into().unwrap(),
                    xkb::KEYMAP_FORMAT_TEXT_V1,
                    xkb::COMPILE_NO_FLAGS,
                )
                .expect("Failed to create keymap")
            }
            .unwrap();

            apply_keymap(kb, keymap);
        }
    }
}

pub fn handle_modifiers(kb: &mut KbState, depressed: u32, latched: u32, locked: u32, group: u32) {
    if let Some(state) = &mut kb.state {
        state.update_mask(depressed, latched, locked, 0, 0, group);
    }
}

pub fn handle_key(kb: &mut KbState, key: u32, ime: &mut ImeEngine) -> bool {
    if let Some(state) = &kb.state {
        let keycode = xkb::Keycode::new(key + 8);

        let sym = state.key_get_one_sym(keycode);

        match sym {
            xkb::Keysym::space => {
                return ime.handle_action(KeyAction::NextCandidate);
            }
            xkb::Keysym::BackSpace => {
                return ime.handle_action(KeyAction::Backspace);
            }
            xkb::Keysym::Return => {
                return ime.handle_action(KeyAction::Confirm);
            }
            xkb::Keysym::Escape => {
                return ime.handle_action(KeyAction::Cancel);
            }
            xkb::Keysym::Up => {
                return ime.handle_action(KeyAction::PrevCandidate);
            }
            xkb::Keysym::Down => {
                return ime.handle_action(KeyAction::NextCandidate);
            }
            xkb::Keysym::Zenkaku_Hankaku => {
                ime.switch_mode();
                return true;
            }
            xkb::Keysym::_1 => {
                return ime.handle_action(KeyAction::SelectCandidate(0));
            }
            xkb::Keysym::_2 => {
                return ime.handle_action(KeyAction::SelectCandidate(1));
            }
            xkb::Keysym::_3 => {
                return ime.handle_action(KeyAction::SelectCandidate(2));
            }
            xkb::Keysym::_4 => {
                return ime.handle_action(KeyAction::SelectCandidate(3));
            }
            xkb::Keysym::_5 => {
                return ime.handle_action(KeyAction::SelectCandidate(4));
            }
            xkb::Keysym::_6 => {
                return ime.handle_action(KeyAction::SelectCandidate(5));
            }
            xkb::Keysym::_7 => {
                return ime.handle_action(KeyAction::SelectCandidate(6));
            }
            xkb::Keysym::_8 => {
                return ime.handle_action(KeyAction::SelectCandidate(7));
            }
            xkb::Keysym::_9 => {
                return ime.handle_action(KeyAction::SelectCandidate(8));
            }
            _ => {
                let text = state.key_get_utf8(keycode);
                if !text.is_empty() && !text.chars().any(|c| c.is_control()) {
                    return ime.handle_action(KeyAction::Insert(text));
                }
            }
        }
    }

    false
}
