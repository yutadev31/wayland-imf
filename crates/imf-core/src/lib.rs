#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    Insert(String),
    Backspace,
    Confirm,
    Cancel,
    NextCandidate,
    PrevCandidate,
    SelectCandidate(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    Conversion,
    Hiragana,
    Katakana,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    text: String,
    kind: CandidateKind,
    score: i32,
}

impl Candidate {
    pub fn new(text: impl Into<String>, kind: CandidateKind, score: i32) -> Self {
        Self {
            text: text.into(),
            kind,
            score,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn kind(&self) -> CandidateKind {
        self.kind
    }

    pub fn score(&self) -> i32 {
        self.score
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompositionState {
    raw_input: String,
    preedit: String,
    candidates: Vec<Candidate>,
    selected_index: Option<usize>,
}

impl CompositionState {
    pub fn raw_input(&self) -> &str {
        &self.raw_input
    }

    pub fn preedit(&self) -> &str {
        &self.preedit
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selected_index
            .and_then(|index| self.candidates.get(index).map(Candidate::text))
    }

    pub fn display_text(&self) -> &str {
        self.selected_text().unwrap_or(&self.preedit)
    }

    pub fn set_raw_input(&mut self, text: String) {
        self.raw_input = text;
    }

    pub fn set_preedit(&mut self, text: String) {
        if self.preedit != text {
            self.selected_index = None;
        }
        self.preedit = text;
    }

    pub fn set_candidates(&mut self, candidates: Vec<Candidate>) {
        self.candidates = candidates;
        if self
            .selected_index
            .is_some_and(|index| index >= self.candidates.len())
        {
            self.selected_index = None;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_index = None;
    }

    pub fn select_index(&mut self, index: usize) -> bool {
        if index >= self.candidates.len() {
            return false;
        }

        self.selected_index = Some(index);
        true
    }

    pub fn select_next(&mut self) -> bool {
        if self.candidates.is_empty() {
            return false;
        }

        self.selected_index = Some(match self.selected_index {
            Some(index) => (index + 1) % self.candidates.len(),
            None => 0,
        });
        true
    }

    pub fn select_previous(&mut self) -> bool {
        if self.candidates.is_empty() {
            return false;
        }

        self.selected_index = Some(match self.selected_index {
            Some(0) | None => self.candidates.len() - 1,
            Some(index) => index - 1,
        });
        true
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    commit_buf: String,
    composition: CompositionState,
}

impl Context {
    pub fn commit_string(&mut self, text: impl AsRef<str>) {
        self.commit_buf.push_str(text.as_ref());
    }

    pub fn take_commit_string(&mut self) -> String {
        std::mem::take(&mut self.commit_buf)
    }

    pub fn composition(&self) -> &CompositionState {
        &self.composition
    }

    pub fn composition_mut(&mut self) -> &mut CompositionState {
        &mut self.composition
    }

    pub fn is_composing(&self) -> bool {
        !self.composition.preedit.is_empty()
    }

    pub fn reset_composition(&mut self) {
        self.composition.clear();
    }
}

pub trait InputMethod {
    fn handle_action(&mut self, ctx: &mut Context, action: KeyAction) -> bool;
}

#[derive(Debug, Default)]
pub struct KeyboardInputMethod;

impl InputMethod for KeyboardInputMethod {
    fn handle_action(&mut self, _ctx: &mut Context, action: KeyAction) -> bool {
        match action {
            KeyAction::Insert(_) => false,
            KeyAction::Backspace
            | KeyAction::Confirm
            | KeyAction::Cancel
            | KeyAction::NextCandidate
            | KeyAction::PrevCandidate
            | KeyAction::SelectCandidate(_) => false,
        }
    }
}
