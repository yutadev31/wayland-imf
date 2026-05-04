#[derive(Debug, Default)]
pub struct ImeState {
    pub buffer: String,
    pub cursor: u16,
    pub commit_pending: bool,
}

impl ImeState {
    pub fn input_char(&mut self, text: String) {
        self.buffer.push_str(&text);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        self.buffer.pop();
        self.cursor -= 1;
    }

    pub fn enter(&mut self) {
        self.commit_pending = true;
    }

    pub fn space(&mut self) {
        if self.buffer.is_empty() {
            self.input_char(" ".into());
            return;
        }

        // TODO 変換処理
    }
}
