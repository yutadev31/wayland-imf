use imf_core::{Context, InputMethod, KeyboardInputMethod};
use japanese_ime::JapaneseInputMethod;

#[derive(Default)]
pub struct ImeState {
    methods: Vec<Box<dyn InputMethod>>,
    current_method: usize,
    last_preedit: String,
    pub context: Context,
}

impl ImeState {
    pub fn init(&mut self) {
        self.methods.push(Box::new(KeyboardInputMethod::default()));
        self.methods.push(Box::new(JapaneseInputMethod::default()));
    }

    pub fn input_char(&mut self, text: String) -> bool {
        self.methods[self.current_method].on_input_str(&mut self.context, text)
    }

    pub fn post_update_preedit(&mut self) {
        let buf = self.context.preedit_buf.clone();
        self.methods[self.current_method].on_update_preedit(&mut self.context, buf);

        if self.context.preedit_buf != self.last_preedit {
            self.context.selected_index = None;
        }

        self.last_preedit = self.context.preedit_buf.clone();
    }

    pub fn backspace(&mut self) -> bool {
        if !self.has_preedit() {
            return false;
        }

        self.context.preedit_buf.pop();
        true
    }

    pub fn enter(&mut self) -> bool {
        if !self.has_preedit() {
            return false;
        }

        let buf = self.current_preedit_text();
        self.context.preedit_buf.clear();
        self.context.commit_string(buf);
        true
    }

    pub fn escape(&mut self) -> bool {
        if self.context.selected_index.is_some() {
            self.context.selected_index = None;
            return true;
        }

        if self.has_preedit() {
            self.context.preedit_buf.clear();
            return true;
        }

        false
    }

    pub fn space(&mut self) -> bool {
        if !self.has_preedit() {
            return false;
        }

        if self.has_candidates() {
            self.select_next_candidate();
        }

        true
    }

    pub fn up(&mut self) -> bool {
        if self.can_select_candidates() {
            self.select_previous_candidate();
            return true;
        }

        false
    }

    pub fn down(&mut self) -> bool {
        if self.can_select_candidates() {
            self.select_next_candidate();
            return true;
        }

        false
    }

    pub fn switch_mode(&mut self) {
        self.reset_composition();
        self.current_method = (self.current_method + 1) % self.methods.len();
    }

    pub fn get_preedit(&self) -> String {
        self.current_preedit_text()
    }

    fn has_preedit(&self) -> bool {
        !self.context.preedit_buf.is_empty()
    }

    fn has_candidates(&self) -> bool {
        !self.context.candidates.is_empty()
    }

    fn can_select_candidates(&self) -> bool {
        self.has_preedit() && self.has_candidates()
    }

    fn reset_composition(&mut self) {
        self.last_preedit.clear();
        self.context.preedit_buf.clear();
        self.context.candidates.clear();
        self.context.selected_index = None;
    }

    fn current_preedit_text(&self) -> String {
        if let Some(index) = self.context.selected_index {
            return self
                .context
                .candidates
                .get(index)
                .map(|text| text.clone())
                .unwrap_or(self.context.preedit_buf.clone());
        }

        self.context.preedit_buf.clone()
    }

    fn select_next_candidate(&mut self) {
        if !self.has_candidates() {
            return;
        }

        self.context.selected_index = Some(match self.context.selected_index {
            Some(index) => (index + 1) % self.context.candidates.len(),
            None => 0,
        });
    }

    fn select_previous_candidate(&mut self) {
        if !self.has_candidates() {
            return;
        }

        self.context.selected_index = Some(match self.context.selected_index {
            Some(0) | None => self.context.candidates.len() - 1,
            Some(index) => index - 1,
        });
    }
}
