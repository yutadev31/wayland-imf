#[derive(Debug, Default)]
pub struct ImeState {
    pub preedit_buf: String,
    pub commit_buf: String,
    pub cursor: u16,
    pub commit_pending: bool,
}

impl ImeState {
    pub fn input_char(&mut self, text: String) {
        self.preedit_buf.push_str(&text);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.preedit_buf.is_empty() {
            return;
        }

        self.preedit_buf.pop();
        self.cursor -= 1;
    }

    pub fn enter(&mut self) {
        self.commit_pending = true;
    }

    pub fn space(&mut self) {
        if self.preedit_buf.is_empty() {
            self.input_char(" ".into());
            return;
        }

        // TODO 変換処理
    }
}
