use std::collections::HashMap;

use ime_core::{Context, InputMethod};

use crate::dict::load_dict;

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

        // =========================
        // ① n（ん）処理
        // =========================
        if chars[i] == 'n' {
            if i + 1 < chars.len() {
                let next = chars[i + 1];

                // nn → ん
                if next == 'n' {
                    result.push('ん');
                    i += 2;
                    continue;
                }

                // n + 子音 → ん
                if !is_vowel(next) && next != 'y' {
                    result.push('ん');
                    i += 1;
                    continue;
                }
            }
        }

        // =========================
        // ② 促音（っ）処理
        // =========================
        if i + 1 < chars.len() {
            let c1 = chars[i];
            let c2 = chars[i + 1];

            if c1 == c2 && is_consonant(c1) {
                result.push('っ');
                i += 1;
                continue;
            }
        }

        // =========================
        // ③ ローマ字辞書変換（最長一致）
        // =========================
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
            // 未変換はそのまま
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

fn generate_candidates(text: &str, dict: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    if let Some(list) = dict.get(text) {
        result.append(&mut list.clone());
    }

    if let Some((stem, okuri)) = hira_to_okuri(text) {
        let key = format!("{}{}", stem, okuri);

        if let Some(list) = dict.get(&key) {
            let okuri_kana = match okuri {
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
            };

            result.append(
                &mut list
                    .iter()
                    .map(|kanji| format!("{}{}", kanji, okuri_kana))
                    .collect(),
            );
        }
    }

    if !text.is_empty() {
        result.push(text.to_string());
        result.push(
            text.chars()
                .filter_map(|ch| {
                    if ch.is_ascii() {
                        Some(ch)
                    } else {
                        char::from_u32((ch as u32) + 0x60)
                    }
                })
                .collect(),
        );
    }

    result
}

#[derive(Debug)]
pub struct JapaneseInputMethod {
    romaji_table: HashMap<&'static str, &'static str>,
    dict: HashMap<String, Vec<String>>,
}

impl Default for JapaneseInputMethod {
    fn default() -> Self {
        Self {
            romaji_table: romaji::romaji_table(),
            dict: load_dict(),
        }
    }
}

impl InputMethod for JapaneseInputMethod {
    fn on_input_str(&mut self, ctx: &mut Context, text: String) -> bool {
        let mut buf = ctx.preedit_buf.clone();
        buf.push_str(&text);

        let buf = to_kana(&self.romaji_table, &buf);
        ctx.set_preedit(buf);
        return true;
    }

    fn on_update_preedit(&mut self, ctx: &mut Context, text: String) {
        let list = generate_candidates(&text, &self.dict);
        ctx.set_candidates(list);
    }
}
