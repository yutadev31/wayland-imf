use imf_core::{Context, InputMethod, KeyAction, KeyboardInputMethod};
use japanese_ime::JapaneseInputMethod;

pub struct ImeEngine {
    methods: Vec<Box<dyn InputMethod>>,
    current_method: usize,
    context: Context,
}

impl ImeEngine {
    pub fn new() -> Self {
        Self {
            methods: vec![
                Box::new(KeyboardInputMethod),
                Box::new(JapaneseInputMethod::default()),
            ],
            current_method: 0,
            context: Context::default(),
        }
    }

    pub fn handle_action(&mut self, action: KeyAction) -> bool {
        self.methods[self.current_method].handle_action(&mut self.context, action)
    }

    pub fn switch_mode(&mut self) {
        self.context.reset_composition();
        self.current_method = (self.current_method + 1) % self.methods.len();
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn display_preedit(&self) -> &str {
        self.context.composition().display_text()
    }
}

impl Default for ImeEngine {
    fn default() -> Self {
        Self::new()
    }
}
