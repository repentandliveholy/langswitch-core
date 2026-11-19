use crate::Lang;

/// Maps a single US/FR-QWERTY character to its corresponding Russian ЙЦУКЕН character.
pub fn qwerty_char_to_jcuken(c: char) -> char {
    match c {
        '`' => 'ё',
        'q' => 'й',
        'w' => 'ц',
        'e' => 'у',
        'r' => 'к',
        't' => 'е',
        'y' => 'н',
        'u' => 'г',
        'i' => 'ш',
        'o' => 'щ',
        'p' => 'з',
        '[' => 'х',
        ']' => 'ъ',
        'a' => 'ф',
        's' => 'ы',
        'd' => 'в',
        'f' => 'а',
        'g' => 'п',
        'h' => 'р',
        'j' => 'о',
        'k' => 'л',
        'l' => 'д',
        ';' => 'ж',
        '\'' => 'э',
        'z' => 'я',
        'x' => 'ч',
        'c' => 'с',
        'v' => 'м',
        'b' => 'и',
        'n' => 'т',
        'm' => 'ь',
        ',' => 'б',
        '.' => 'ю',
        '/' => '.',
        '~' => 'Ё',
        'Q' => 'Й',
        'W' => 'Ц',
        'E' => 'У',
        'R' => 'К',
        'T' => 'Е',
        'Y' => 'Н',
        'U' => 'Г',
        'I' => 'Ш',
        'O' => 'Щ',
        'P' => 'З',
        '{' => 'Х',
        '}' => 'Ъ',
        'A' => 'Ф',
        'S' => 'Ы',
        'D' => 'В',
        'F' => 'А',
        'G' => 'П',
        'H' => 'Р',
        'J' => 'О',
        'K' => 'Л',
        'L' => 'Д',
        ':' => 'Ж',
        '"' => 'Э',
        'Z' => 'Я',
        'X' => 'Ч',
        'C' => 'С',
        'V' => 'М',
        'B' => 'И',
        'N' => 'Т',
        'M' => 'Ь',
        '<' => 'Б',
        '>' => 'Ю',
        '?' => ',',
        '@' => '"',
        '#' => '№',
        '$' => ';',
        '^' => ':',
        '&' => '?',
        other => other,
    }
}

/// Maps a single Russian ЙЦУКЕН character to its corresponding US/FR-QWERTY character.
pub fn jcuken_char_to_qwerty(c: char) -> char {
    match c {
        'ё' => '`',
        'й' => 'q',
        'ц' => 'w',
        'у' => 'e',
        'к' => 'r',
        'е' => 't',
        'н' => 'y',
        'г' => 'u',
        'ш' => 'i',
        'щ' => 'o',
        'з' => 'p',
        'х' => '[',
        'ъ' => ']',
        'ф' => 'a',
        'ы' => 's',
        'в' => 'd',
        'а' => 'f',
        'п' => 'g',
        'р' => 'h',
        'о' => 'j',
        'л' => 'k',
        'д' => 'l',
        'ж' => ';',
        'э' => '\'',
        'я' => 'z',
        'ч' => 'x',
        'с' => 'c',
        'м' => 'v',
        'и' => 'b',
        'т' => 'n',
        'ь' => 'm',
        'б' => ',',
        'ю' => '.',
        'Ё' => '~',
        'Й' => 'Q',
        'Ц' => 'W',
        'У' => 'E',
        'К' => 'R',
        'Е' => 'T',
        'Н' => 'Y',
        'Г' => 'U',
        'Ш' => 'I',
        'Щ' => 'O',
        'З' => 'P',
        'Х' => '{',
        'Ъ' => '}',
        'Ф' => 'A',
        'Ы' => 'S',
        'В' => 'D',
        'А' => 'F',
        'П' => 'G',
        'Р' => 'H',
        'О' => 'J',
        'Л' => 'K',
        'Д' => 'L',
        'Ж' => ':',
        'Э' => '"',
        'Я' => 'Z',
        'Ч' => 'X',
        'С' => 'C',
        'М' => 'V',
        'И' => 'B',
        'Т' => 'N',
        'Ь' => 'M',
        'Б' => '<',
        'Ю' => '>',
        '№' => '#',
        other => other,
    }
}

/// Maps a single French AZERTY character to its corresponding Russian ЙЦУКЕН character.
/// On French AZERTY:
/// Row 2: a->й, z->ц, e->у, r->к, t->е, y->н, u->г, i->ш, o->щ, p->з, ^->х, $->ъ
/// Row 3: q->ф, s->ы, d->в, f->а, g->п, h->р, j->о, k->л, l->д, m->ж, ù->э
/// Row 4: w->я, x->ч, c->с, v->м, b->и, n->т, ,->ь, ;->б, :->ю
pub fn azerty_char_to_jcuken(c: char) -> char {
    match c {
        'a' => 'й',
        'z' => 'ц',
        'e' => 'у',
        'r' => 'к',
        't' => 'е',
        'y' => 'н',
        'u' => 'г',
        'i' => 'ш',
        'o' => 'щ',
        'p' => 'з',
        '^' => 'х',
        '$' => 'ъ',
        'q' => 'ф',
        's' => 'ы',
        'd' => 'в',
        'f' => 'а',
        'g' => 'п',
        'h' => 'р',
        'j' => 'о',
        'k' => 'л',
        'l' => 'д',
        'm' => 'ж',
        'ù' => 'э',
        'w' => 'я',
        'x' => 'ч',
        'c' => 'с',
        'v' => 'м',
        'b' => 'и',
        'n' => 'т',
        ',' => 'ь',
        ';' => 'б',
        ':' => 'ю',
        'A' => 'Й',
        'Z' => 'Ц',
        'E' => 'У',
        'R' => 'К',
        'T' => 'Е',
        'Y' => 'Н',
        'U' => 'Г',
        'I' => 'Ш',
        'O' => 'Щ',
        'P' => 'З',
        'Q' => 'Ф',
        'S' => 'Ы',
        'D' => 'В',
        'F' => 'А',
        'G' => 'П',
        'H' => 'Р',
        'J' => 'О',
        'K' => 'Л',
        'L' => 'Д',
        'M' => 'Ж',
        'W' => 'Я',
        'X' => 'Ч',
        'C' => 'С',
        'V' => 'М',
        'B' => 'И',
        'N' => 'Т',
        '?' => 'Ь',
        '.' => 'Б',
        '/' => 'Ю',
        other => qwerty_char_to_jcuken(other),
    }
}

/// Maps a single Russian ЙЦУКЕН character to its corresponding French AZERTY character.
pub fn jcuken_char_to_azerty(c: char) -> char {
    match c {
        'й' => 'a',
        'ц' => 'z',
        'у' => 'e',
        'к' => 'r',
        'е' => 't',
        'н' => 'y',
        'г' => 'u',
        'ш' => 'i',
        'щ' => 'o',
        'з' => 'p',
        'х' => '^',
        'ъ' => '$',
        'ф' => 'q',
        'ы' => 's',
        'в' => 'd',
        'а' => 'f',
        'п' => 'g',
        'р' => 'h',
        'о' => 'j',
        'л' => 'k',
        'д' => 'l',
        'ж' => 'm',
        'э' => 'ù',
        'я' => 'w',
        'ч' => 'x',
        'с' => 'c',
        'м' => 'v',
        'и' => 'b',
        'т' => 'n',
        'ь' => ',',
        'б' => ';',
        'ю' => ':',
        'Й' => 'A',
        'Ц' => 'Z',
        'У' => 'E',
        'К' => 'R',
        'Е' => 'T',
        'Н' => 'Y',
        'Г' => 'U',
        'Ш' => 'I',
        'Щ' => 'O',
        'З' => 'P',
        'Ф' => 'Q',
        'Ы' => 'S',
        'В' => 'D',
        'А' => 'F',
        'П' => 'G',
        'Р' => 'H',
        'О' => 'J',
        'Л' => 'K',
        'Д' => 'L',
        'Ж' => 'M',
        'Я' => 'W',
        'Ч' => 'X',
        'С' => 'C',
        'М' => 'V',
        'И' => 'B',
        'Т' => 'N',
        'Ь' => '?',
        'Б' => '.',
        'Ю' => '/',
        other => jcuken_char_to_qwerty(other),
    }
}

/// Maps a single Turkish-Q character to its corresponding Russian ЙЦУКЕН character.
/// On Turkish-Q:
/// `ğ` is on `[` (`х`), `ü` is on `]` (`ъ`), `ş` is on `;` (`ж`), `i` is on `'` (`э`),
/// `ı` is on `i` (`ш`), `ö` is on `,` (`б`), `ç` is on `.` (`ю`).
pub fn turkish_q_char_to_jcuken(c: char) -> char {
    match c {
        'ğ' => 'х',
        'ü' => 'ъ',
        'ş' => 'ж',
        'ı' => 'ш',
        'ö' => 'б',
        'ç' => 'ю',
        'Ğ' => 'Х',
        'Ü' => 'Ъ',
        'Ş' => 'Ж',
        'I' => 'Ш',
        'İ' => 'Э',
        'Ö' => 'Б',
        'Ç' => 'Ю',
        other => qwerty_char_to_jcuken(other),
    }
}

/// Maps a single Russian ЙЦУКЕН character to its corresponding Turkish-Q character.
pub fn jcuken_char_to_turkish_q(c: char) -> char {
    match c {
        'х' => 'ğ',
        'ъ' => 'ü',
        'ж' => 'ş',
        'э' => 'i',
        'ш' => 'ı',
        'б' => 'ö',
        'ю' => 'ç',
        'Х' => 'Ğ',
        'Ъ' => 'Ü',
        'Ж' => 'Ş',
        'Э' => 'İ',
        'Ш' => 'I',
        'Б' => 'Ö',
        'Ю' => 'Ç',
        other => jcuken_char_to_qwerty(other),
    }
}

pub fn qwerty_to_jcuken(s: &str) -> String {
    s.chars().map(qwerty_char_to_jcuken).collect()
}

pub fn jcuken_to_qwerty(s: &str) -> String {
    s.chars().map(jcuken_char_to_qwerty).collect()
}

pub fn azerty_to_jcuken(s: &str) -> String {
    s.chars().map(azerty_char_to_jcuken).collect()
}

pub fn jcuken_to_azerty(s: &str) -> String {
    s.chars().map(jcuken_char_to_azerty).collect()
}

pub fn turkish_q_to_jcuken(s: &str) -> String {
    s.chars().map(turkish_q_char_to_jcuken).collect()
}

pub fn jcuken_to_turkish_q(s: &str) -> String {
    s.chars().map(jcuken_char_to_turkish_q).collect()
}

/// Converts arbitrary text (including multi-word selections) to the target layout.
pub fn convert_text_layout(s: &str, current_lang: Lang, target_lang: Lang) -> String {
    match (current_lang, target_lang) {
        (Lang::En, Lang::Ru) => qwerty_to_jcuken(s),
        (Lang::Ru, Lang::En) => jcuken_to_qwerty(s),
        (Lang::Fr, Lang::Ru) => qwerty_to_jcuken(s),
        (Lang::Ru, Lang::Fr) => jcuken_to_qwerty(s),
        (Lang::Tr, Lang::Ru) => turkish_q_to_jcuken(s),
        (Lang::Ru, Lang::Tr) => jcuken_to_turkish_q(s),
        _ => qwerty_to_jcuken(s),
    }
}

/// Detects accidental Caps Lock typing such as `hELLO` -> `Hello` or `пРИВЕТ` -> `Привет`.
pub fn detect_inverted_caps(word: &str) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < 3 {
        return None;
    }

    let first = chars[0];
    let rest = &chars[1..];

    let first_is_lower = first.is_lowercase() && first.is_alphabetic();
    let rest_all_upper = rest
        .iter()
        .all(|c| c.is_uppercase() && c.is_alphabetic());

    if first_is_lower && rest_all_upper {
        let mut fixed = String::with_capacity(word.len());
        for c in first.to_uppercase() {
            fixed.push(c);
        }
        for &c in rest {
            for lc in c.to_lowercase() {
                fixed.push(lc);
            }
        }
        Some(fixed)
    } else {
        None
    }
}
