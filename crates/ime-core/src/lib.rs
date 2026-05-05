#[derive(Debug, Default)]
pub struct Context {
    pub commit_buf: String,
    pub preedit_buf: String,
}

impl Context {
    pub fn commit_string(&mut self, text: String) {
        self.commit_buf.push_str(&text);
    }

    pub fn set_preedit(&mut self, text: String) {
        self.preedit_buf = text;
    }

    // pub fn set_candidates(&mut self, list: Vec<String>) {}
}

pub trait InputMethod {
    fn on_input_str(&mut self, ctx: &mut Context, text: String) -> bool;
}

#[derive(Debug, Default)]
pub struct KeyboardInputMethod {}

impl InputMethod for KeyboardInputMethod {
    fn on_input_str(&mut self, _ctx: &mut Context, _text: String) -> bool {
        return false;
    }
}
