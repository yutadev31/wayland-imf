use std::collections::HashMap;

fn roma_table() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("a", "あ"),
        ("i", "い"),
        ("u", "う"),
        ("e", "え"),
        ("o", "お"),
        ("ka", "か"),
        ("ki", "き"),
        ("ku", "く"),
        ("ke", "け"),
        ("ko", "こ"),
        ("kya", "きゃ"),
        ("kyu", "きゅ"),
        ("kyo", "きょ"),
        ("ga", "が"),
        ("gi", "ぎ"),
        ("gu", "ぐ"),
        ("ge", "げ"),
        ("go", "ご"),
        ("gya", "ぎゃ"),
        ("gyu", "ぎゅ"),
        ("gyo", "ぎょ"),
        ("sa", "さ"),
        ("si", "し"),
        ("su", "す"),
        ("se", "せ"),
        ("so", "そ"),
        ("sya", "しゃ"),
        ("syu", "しゅ"),
        ("syo", "しょ"),
        ("sha", "しゃ"),
        ("shi", "し"),
        ("shu", "しゅ"),
        ("she", "しぇ"),
        ("sho", "しょ"),
        ("za", "ざ"),
        ("zi", "じ"),
        ("zu", "ず"),
        ("ze", "ぜ"),
        ("zo", "ぞ"),
        ("zya", "じゃ"),
        ("zyu", "じゅ"),
        ("zyo", "じょ"),
        ("ta", "た"),
        ("ti", "ち"),
        ("tu", "つ"),
        ("te", "て"),
        ("to", "と"),
        ("tya", "ちゃ"),
        ("tyu", "ちゅ"),
        ("tyo", "ちょ"),
        ("da", "だ"),
        ("di", "ぢ"),
        ("du", "づ"),
        ("de", "で"),
        ("do", "ど"),
        ("dya", "ぢゃ"),
        ("dyu", "ぢゅ"),
        ("dyo", "ぢょ"),
        ("cha", "ちゃ"),
        ("chi", "ち"),
        ("chu", "ちゅ"),
        ("che", "ちぇ"),
        ("cho", "ちょ"),
        ("tsu", "つ"),
        ("na", "な"),
        ("ni", "に"),
        ("nu", "ぬ"),
        ("ne", "ね"),
        ("no", "の"),
        ("nya", "にゃ"),
        ("nyu", "にゅ"),
        ("nyo", "にょ"),
        ("ha", "は"),
        ("hi", "ひ"),
        ("fu", "ふ"),
        ("he", "へ"),
        ("ho", "ほ"),
        ("hya", "ひゃ"),
        ("hyu", "ひゅ"),
        ("hyo", "ひょ"),
        ("ba", "ば"),
        ("bi", "び"),
        ("bu", "ぶ"),
        ("be", "べ"),
        ("bo", "ぼ"),
        ("bya", "びゃ"),
        ("byu", "びゅ"),
        ("byo", "びょ"),
        ("pa", "ぱ"),
        ("pi", "ぴ"),
        ("pu", "ぷ"),
        ("pe", "ぺ"),
        ("po", "ぽ"),
        ("pya", "ぴゃ"),
        ("pyu", "ぴゅ"),
        ("pyo", "ぴょ"),
        ("ma", "ま"),
        ("mi", "み"),
        ("mu", "む"),
        ("me", "め"),
        ("mo", "も"),
        ("mya", "みゃ"),
        ("myu", "みゅ"),
        ("myo", "みょ"),
        ("ya", "や"),
        ("yu", "ゆ"),
        ("ye", "いぇ"),
        ("yo", "よ"),
        ("ra", "ら"),
        ("ri", "り"),
        ("ru", "る"),
        ("re", "れ"),
        ("ro", "ろ"),
        ("rya", "りゃ"),
        ("ryu", "りゅ"),
        ("ryo", "りょ"),
        ("wa", "わ"),
        ("wi", "うぃ"),
        ("we", "うぇ"),
        ("wo", "を"),
        ("n", "ん"),
        ("va", "ゔぁ"),
        ("vi", "ゔぃ"),
        ("vu", "ゔ"),
        ("ve", "ゔぇ"),
        ("vo", "ゔぉ"),
        ("ltu", "っ"),
        ("la", "ぁ"),
        ("li", "ぃ"),
        ("lu", "ぅ"),
        ("le", "ぇ"),
        ("lo", "ぉ"),
    ])
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'i' | 'u' | 'e' | 'o')
}

fn is_consonant(c: char) -> bool {
    c.is_ascii_alphabetic() && !is_vowel(c)
}

#[derive(Debug, Default)]
pub struct ImeState {
    pub preedit_raw: String,
    pub preedit_kana: String,
    pub commit_buf: String,
    pub commit_pending: bool,
}

impl ImeState {
    pub fn input_char(&mut self, text: String) {
        self.preedit_raw.push_str(&text);
        self.recompute();
    }

    pub fn backspace(&mut self) {
        if self.preedit_raw.is_empty() {
            return;
        }

        self.preedit_raw.pop();
        self.recompute();
    }

    pub fn enter(&mut self) {
        self.commit_pending = true;
    }

    pub fn space(&mut self) {
        if self.preedit_raw.is_empty() {
            self.input_char(" ".into());
            return;
        }

        // TODO 変換処理
    }

    pub fn recompute(&mut self) {
        let table = roma_table();
        let chars: Vec<char> = self.preedit_raw.chars().collect();
        let mut result = String::new();

        let mut i = 0;
        while i < chars.len() {
            // -----------------
            // ① 促音（っ）
            // -----------------
            if i + 1 < chars.len()
                && chars[i] == chars[i + 1]
                && is_consonant(chars[i])
                && chars[i] != 'n'
            {
                result.push_str("っ");
                i += 1;
                continue;
            }

            // -----------------
            // ② 「ん」処理
            // -----------------
            if chars[i] == 'n' {
                if i + 1 < chars.len() {
                    let next = chars[i + 1];

                    // nn → ん
                    if next == 'n' {
                        result.push_str("ん");
                        i += 2;
                        continue;
                    }

                    // n + 子音（母音以外）→ ん
                    if !is_vowel(next) && next != 'y' {
                        result.push_str("ん");
                        i += 1;
                        continue;
                    }

                    // n + 母音 → そのまま（na, ni など）
                } else {
                    // 末尾の n → ん
                    result.push_str("ん");
                    i += 1;
                    continue;
                }
            }

            // -----------------
            // ③ 通常変換（最大3文字）
            // -----------------
            let mut matched = false;

            for len in (1..=3).rev() {
                if i + len > chars.len() {
                    continue;
                }

                let s: String = chars[i..i + len].iter().collect();

                if let Some(h) = table.get(s.as_str()) {
                    result.push_str(h);
                    i += len;
                    matched = true;
                    break;
                }
            }

            // -----------------
            // ④ 未確定文字
            // -----------------
            if !matched {
                result.push(chars[i]);
                i += 1;
            }
        }

        self.preedit_kana = result;
    }
}
