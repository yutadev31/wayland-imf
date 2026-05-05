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
    pub context: Context,
}

impl ImeState {
    pub fn input_char(&mut self, text: String) -> bool {
        return match &mut self.method {
            InputMethodEnum::Keyboard(method) => method.on_input_str(&mut self.context, text),
            InputMethodEnum::Japanese(method) => method.on_input_str(&mut self.context, text),
        };
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

        let buf = self.context.preedit_buf.clone();
        self.context.preedit_buf.clear();
        self.context.commit_buf.push_str(&buf);
        return true;
    }

    pub fn space(&mut self) {
        if self.context.preedit_buf.is_empty() {
            self.input_char(" ".into());
            return;
        }

        // TODO 変換処理
    }

    pub fn switch_mode(&mut self) {
        self.method = match self.method {
            InputMethodEnum::Keyboard(_) => {
                InputMethodEnum::Japanese(JapaneseInputMethod::default())
            }
            InputMethodEnum::Japanese(_) => {
                InputMethodEnum::Keyboard(KeyboardInputMethod::default())
            }
        };
    }
}
