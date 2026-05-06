use ime_core::{Context, InputMethod, KeyboardInputMethod};
use ja_im::JapaneseInputMethod;

#[derive(Debug)]
pub enum InputMethodEnum {
    Keyboard(KeyboardInputMethod),
    Japanese(JapaneseInputMethod),
}

impl Default for InputMethodEnum {
    fn default() -> Self {
        InputMethodEnum::Keyboard(KeyboardInputMethod::default())
    }
}

#[derive(Debug, Default)]
pub struct ImeState {
    method: InputMethodEnum,
    last_preedit: String,
    pub context: Context,
}

impl ImeState {
    pub fn input_char(&mut self, text: String) -> bool {
        return match &mut self.method {
            InputMethodEnum::Keyboard(method) => method.on_input_str(&mut self.context, text),
            InputMethodEnum::Japanese(method) => method.on_input_str(&mut self.context, text),
        };
    }

    pub fn post_update_preedit(&mut self) {
        let buf = self.context.preedit_buf.clone();
        match &mut self.method {
            InputMethodEnum::Keyboard(method) => method.on_update_preedit(&mut self.context, buf),
            InputMethodEnum::Japanese(method) => method.on_update_preedit(&mut self.context, buf),
        }

        if self.context.preedit_buf != self.last_preedit {
            self.context.selected_index = None;
        }

        self.last_preedit = self.context.preedit_buf.clone();
    }

    pub fn backspace(&mut self) -> bool {
        if self.context.preedit_buf.is_empty() {
            return false;
        }

        self.context.preedit_buf.pop();
        return true;
    }

    pub fn enter(&mut self) -> bool {
        if self.context.preedit_buf.is_empty() {
            return false;
        }

        let buf = self.get_preedit();
        self.context.preedit_buf.clear();
        self.context.commit_buf.push_str(&buf);
        return true;
    }

    pub fn escape(&mut self) -> bool {
        if let Some(_) = self.context.selected_index {
            self.context.selected_index = None;
            return true;
        }

        if !self.context.preedit_buf.is_empty() {
            self.context.preedit_buf.clear();
            return true;
        }

        return false;
    }

    pub fn space(&mut self) -> bool {
        if self.context.preedit_buf.is_empty() {
            return false;
        }

        if !self.context.candidates.is_empty() {
            match self.context.selected_index {
                Some(index) => {
                    self.context.selected_index = Some((index + 1) % self.context.candidates.len());
                }
                None => {
                    self.context.selected_index = Some(0);
                }
            }
        }

        return true;
    }

    pub fn up(&mut self) -> bool {
        if !self.context.preedit_buf.is_empty() && !self.context.candidates.is_empty() {
            match self.context.selected_index {
                Some(index) => {
                    if index == 0 {
                        self.context.selected_index = Some(self.context.candidates.len() - 1);
                    } else {
                        self.context.selected_index = Some(index - 1);
                    }
                }
                None => {
                    self.context.selected_index = Some(self.context.candidates.len() - 1);
                }
            }
            return true;
        }

        return false;
    }

    pub fn down(&mut self) -> bool {
        if !self.context.preedit_buf.is_empty() && !self.context.candidates.is_empty() {
            match self.context.selected_index {
                Some(index) => {
                    self.context.selected_index = Some((index + 1) % self.context.candidates.len());
                }
                None => {
                    self.context.selected_index = Some(0);
                }
            }
            return true;
        }

        return false;
    }

    pub fn switch_mode(&mut self) {
        self.last_preedit.clear();
        self.context.preedit_buf.clear();
        self.context.candidates.clear();

        self.method = match self.method {
            InputMethodEnum::Keyboard(_) => {
                InputMethodEnum::Japanese(JapaneseInputMethod::default())
            }
            InputMethodEnum::Japanese(_) => {
                InputMethodEnum::Keyboard(KeyboardInputMethod::default())
            }
        };
    }

    pub fn get_preedit(&self) -> String {
        if let Some(index) = self.context.selected_index {
            let candidates = self.context.candidates.clone();
            return candidates
                .get(index)
                .map(|text| text.clone())
                .unwrap_or(self.context.preedit_buf.clone());
        }

        return self.context.preedit_buf.clone();
    }
}
