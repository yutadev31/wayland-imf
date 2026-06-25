use xkbcommon::xkb;

use crate::{
    ime::ImeEngine,
    keyboard::{KbState, KeyboardConfig},
    ui::{CandidateViewModel, UiState},
    wayland::WaylandState,
};
use wayland_client::QueueHandle;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub keyboard: KeyboardConfig,
}

pub struct State {
    pub wayland: WaylandState,

    pub kb: KbState,
    pub ime: ImeEngine,
    pub ui: UiState,
}

impl State {
    pub fn new(config: Config) -> Self {
        State {
            wayland: WaylandState::default(),
            kb: KbState {
                context: xkb::Context::new(xkb::CONTEXT_NO_FLAGS),
                keymap: None,
                state: None,
                config: config.keyboard,
            },
            ime: ImeEngine::default(),
            ui: UiState::default(),
        }
    }

    pub fn ensure_candidate_popup(&mut self, qh: &QueueHandle<Self>) {
        if self.ui.popup.is_some() {
            return;
        }

        let Some(compositor) = self.wayland.compositor.as_ref() else {
            return;
        };
        let Some(shm) = self.wayland.shm.as_ref() else {
            return;
        };
        let Some(input_method) = self.wayland.input_method.as_ref() else {
            return;
        };

        self.ui.popup = crate::ui::CandidatePopup::new(compositor, shm, input_method, qh).ok();
    }

    pub fn refresh_candidate_popup(&mut self) {
        if let Some(popup) = &mut self.ui.popup {
            let view = CandidateViewModel::from_context(self.ime.context());
            popup.render(view.as_ref());
        }
    }

    pub fn hide_candidate_popup(&mut self) {
        if let Some(popup) = &mut self.ui.popup {
            popup.hide();
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
