use std::collections::{HashMap, HashSet};

use imf_core::{Candidate, CandidateKind, Context, InputMethod, KeyAction};

use crate::dict::{CandidateProvider, StaticDictionary};

mod dict;
mod romaji;

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'i' | 'u' | 'e' | 'o')
}

fn is_consonant(c: char) -> bool {
    c.is_ascii_alphabetic() && !is_vowel(c) && c != 'n'
}

fn to_kana(table: &HashMap<&'static str, &'static str>, input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut result = String::new();

    while i < chars.len() {
        let mut matched: Option<(usize, &str)> = None;

        if chars[i] == 'n' && i + 1 < chars.len() {
            let next = chars[i + 1];

            if next == 'n' {
                result.push('ん');
                i += 2;
                continue;
            }

            if !is_vowel(next) && next != 'y' {
                result.push('ん');
                i += 1;
                continue;
            }
        }

        if i + 1 < chars.len() {
            let c1 = chars[i];
            let c2 = chars[i + 1];

            if c1 == c2 && is_consonant(c1) {
                result.push('っ');
                i += 1;
                continue;
            }
        }

        for len in (1..=4).rev() {
            if i + len > chars.len() {
                continue;
            }

            let slice: String = chars[i..i + len].iter().collect();
            if let Some(&kana) = table.get(slice.as_str()) {
                matched = Some((len, kana));
                break;
            }
        }

        if let Some((len, kana)) = matched {
            result.push_str(kana);
            i += len;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn hira_to_okuri(input: &str) -> Option<(String, char)> {
    let table = [
        ("う", 'u'),
        ("い", 'i'),
        ("く", 'k'),
        ("す", 's'),
        ("つ", 't'),
        ("ぬ", 'n'),
        ("む", 'm'),
        ("る", 'r'),
        ("ぐ", 'g'),
        ("ぶ", 'b'),
    ];

    for (kana, key) in table {
        if input.ends_with(kana) {
            let stem = input.trim_end_matches(kana).to_string();
            return Some((stem, key));
        }
    }

    None
}

fn okuri_kana(okuri: char) -> &'static str {
    match okuri {
        'u' => "う",
        'i' => "い",
        'k' => "く",
        's' => "す",
        't' => "つ",
        'n' => "ぬ",
        'm' => "む",
        'r' => "る",
        'g' => "ぐ",
        'b' => "ぶ",
        _ => "",
    }
}

fn katakana(text: &str) -> String {
    text.chars()
        .filter_map(|ch| {
            if ch.is_ascii() || matches!(ch, 'ー' | '〜' | '、' | '。') {
                Some(ch)
            } else {
                char::from_u32((ch as u32) + 0x60)
            }
        })
        .collect()
}

pub struct JapaneseInputMethod {
    romaji_table: HashMap<&'static str, &'static str>,
    provider: Box<dyn CandidateProvider>,
    history: HashMap<String, HashMap<String, u32>>,
}

impl JapaneseInputMethod {
    pub fn new(provider: Box<dyn CandidateProvider>) -> Self {
        Self {
            romaji_table: romaji::romaji_table(),
            provider,
            history: HashMap::new(),
        }
    }

    fn update_composition(&self, ctx: &mut Context, raw_input: String) {
        let preedit = to_kana(&self.romaji_table, &raw_input);
        let candidates = self.generate_candidates(&preedit);

        let composition = ctx.composition_mut();
        composition.set_raw_input(raw_input);
        composition.set_preedit(preedit);
        composition.set_candidates(candidates);
    }

    fn generate_candidates(&self, text: &str) -> Vec<Candidate> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for candidate in self.provider.candidates_for(text) {
            if seen.insert(candidate.clone()) {
                result.push(Candidate::new(candidate, CandidateKind::Conversion, 1000));
            }
        }

        if let Some((stem, okuri)) = hira_to_okuri(text) {
            let key = format!("{}{}", stem, okuri);
            let suffix = okuri_kana(okuri);
            for kanji in self.provider.candidates_for(&key) {
                let candidate = format!("{}{}", kanji, suffix);
                if seen.insert(candidate.clone()) {
                    result.push(Candidate::new(candidate, CandidateKind::Conversion, 1100));
                }
            }
        }

        if !text.is_empty() {
            let hiragana = text.to_string();
            if seen.insert(hiragana.clone()) {
                result.push(Candidate::new(hiragana, CandidateKind::Hiragana, -100));
            }

            let katakana = katakana(text);
            if seen.insert(katakana.clone()) {
                result.push(Candidate::new(katakana, CandidateKind::Katakana, -200));
            }
        }

        let history = self.history.get(text);
        result.sort_by(|left, right| {
            let left_history = history
                .and_then(|entries| entries.get(left.text()))
                .copied()
                .unwrap_or(0) as i32;
            let right_history = history
                .and_then(|entries| entries.get(right.text()))
                .copied()
                .unwrap_or(0) as i32;

            (right.score() + right_history * 100).cmp(&(left.score() + left_history * 100))
        });

        result
    }

    fn record_selection(&mut self, reading: &str, text: &str) {
        let entries = self.history.entry(reading.to_string()).or_default();
        *entries.entry(text.to_string()).or_default() += 1;
    }
}

impl Default for JapaneseInputMethod {
    fn default() -> Self {
        let provider = StaticDictionary::load("tmp/SKK-JISYO.L")
            .map(|dict| Box::new(dict) as Box<dyn CandidateProvider>)
            .unwrap_or_else(|_| Box::new(StaticDictionary::empty()));

        Self::new(provider)
    }
}

impl InputMethod for JapaneseInputMethod {
    fn handle_action(&mut self, ctx: &mut Context, action: KeyAction) -> bool {
        match action {
            KeyAction::Insert(text) => {
                let raw_input = format!("{}{}", ctx.composition().raw_input(), text);
                self.update_composition(ctx, raw_input);
                true
            }
            KeyAction::Backspace => {
                if ctx.composition().raw_input().is_empty() {
                    return false;
                }

                let mut raw_input = ctx.composition().raw_input().to_string();
                raw_input.pop();

                if raw_input.is_empty() {
                    ctx.reset_composition();
                } else {
                    self.update_composition(ctx, raw_input);
                }
                true
            }
            KeyAction::Confirm => {
                if !ctx.is_composing() {
                    return false;
                }

                let reading = ctx.composition().preedit().to_string();
                let text = ctx.composition().display_text().to_string();
                self.record_selection(&reading, &text);
                ctx.commit_string(text);
                ctx.reset_composition();
                true
            }
            KeyAction::Cancel => {
                if ctx.composition().selected_index().is_some() {
                    ctx.composition_mut().clear_selection();
                    return true;
                }

                if ctx.is_composing() {
                    ctx.reset_composition();
                    return true;
                }

                false
            }
            KeyAction::NextCandidate => {
                if !ctx.is_composing() {
                    return false;
                }

                ctx.composition_mut().select_next()
            }
            KeyAction::PrevCandidate => {
                if !ctx.is_composing() {
                    return false;
                }

                ctx.composition_mut().select_previous()
            }
            KeyAction::SelectCandidate(index) => {
                if !ctx.is_composing() || !ctx.composition_mut().select_index(index) {
                    return false;
                }

                let reading = ctx.composition().preedit().to_string();
                let text = ctx.composition().display_text().to_string();
                self.record_selection(&reading, &text);
                ctx.commit_string(text);
                ctx.reset_composition();
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use imf_core::Context;

    use super::*;

    struct TestProvider {
        entries: HashMap<String, Vec<String>>,
    }

    impl CandidateProvider for TestProvider {
        fn candidates_for(&self, text: &str) -> Vec<String> {
            self.entries.get(text).cloned().unwrap_or_default()
        }
    }

    fn ime_with_entries(entries: &[(&str, &[&str])]) -> JapaneseInputMethod {
        let entries = entries
            .iter()
            .map(|(key, values)| {
                (
                    (*key).to_string(),
                    values.iter().map(|value| (*value).to_string()).collect(),
                )
            })
            .collect();

        JapaneseInputMethod::new(Box::new(TestProvider { entries }))
    }

    #[test]
    fn backspace_uses_raw_romaji_input() {
        let mut ime = ime_with_entries(&[]);
        let mut ctx = Context::default();

        assert!(ime.handle_action(&mut ctx, KeyAction::Insert("k".to_string())));
        assert!(ime.handle_action(&mut ctx, KeyAction::Insert("a".to_string())));
        assert_eq!(ctx.composition().raw_input(), "ka");
        assert_eq!(ctx.composition().preedit(), "か");

        assert!(ime.handle_action(&mut ctx, KeyAction::Backspace));
        assert_eq!(ctx.composition().raw_input(), "k");
        assert_eq!(ctx.composition().preedit(), "k");
    }

    #[test]
    fn confirm_commits_selected_candidate() {
        let mut ime = ime_with_entries(&[("かな", &["仮名", "かな"])]);
        let mut ctx = Context::default();

        assert!(ime.handle_action(&mut ctx, KeyAction::Insert("k".to_string())));
        assert!(ime.handle_action(&mut ctx, KeyAction::Insert("a".to_string())));
        assert!(ime.handle_action(&mut ctx, KeyAction::Insert("n".to_string())));
        assert!(ime.handle_action(&mut ctx, KeyAction::Insert("a".to_string())));
        assert!(ime.handle_action(&mut ctx, KeyAction::NextCandidate));
        assert!(ime.handle_action(&mut ctx, KeyAction::Confirm));

        assert_eq!(ctx.take_commit_string(), "仮名");
        assert!(!ctx.is_composing());
    }

    #[test]
    fn duplicate_fallback_candidates_are_removed() {
        let ime = ime_with_entries(&[("かな", &["かな", "カナ", "仮名"])]);

        let candidates = ime.generate_candidates("かな");
        let texts: Vec<_> = candidates
            .iter()
            .map(|candidate| candidate.text())
            .collect();

        assert_eq!(texts, vec!["かな", "カナ", "仮名"]);
        assert_eq!(texts.len(), 3);
    }

    #[test]
    fn confirmed_candidate_moves_to_front() {
        let mut ime = ime_with_entries(&[("かな", &["仮名", "佳名"])]);
        let mut ctx = Context::default();

        for ch in ["k", "a", "n", "a"] {
            assert!(ime.handle_action(&mut ctx, KeyAction::Insert(ch.to_string())));
        }
        assert!(ime.handle_action(&mut ctx, KeyAction::SelectCandidate(1)));
        assert_eq!(ctx.take_commit_string(), "佳名");

        let candidates = ime.generate_candidates("かな");
        assert_eq!(candidates[0].text(), "佳名");
    }

    #[test]
    fn select_candidate_commits_by_index() {
        let mut ime = ime_with_entries(&[("かな", &["仮名", "佳名"])]);
        let mut ctx = Context::default();

        for ch in ["k", "a", "n", "a"] {
            assert!(ime.handle_action(&mut ctx, KeyAction::Insert(ch.to_string())));
        }

        assert!(ime.handle_action(&mut ctx, KeyAction::SelectCandidate(1)));
        assert_eq!(ctx.take_commit_string(), "佳名");
        assert!(!ctx.is_composing());
    }

    #[test]
    fn cancel_clears_selection_before_composition() {
        let mut ime = ime_with_entries(&[("かな", &["仮名"])]);
        let mut ctx = Context::default();

        for ch in ["k", "a", "n", "a"] {
            assert!(ime.handle_action(&mut ctx, KeyAction::Insert(ch.to_string())));
        }
        assert!(ime.handle_action(&mut ctx, KeyAction::NextCandidate));
        assert!(ctx.composition().selected_index().is_some());

        assert!(ime.handle_action(&mut ctx, KeyAction::Cancel));
        assert_eq!(ctx.composition().selected_index(), None);
        assert!(ctx.is_composing());
    }
}
