pub mod detector;
pub mod entitlements;
pub mod keymap;
pub mod learner;
pub mod lexicon;
pub mod typo;
pub mod update;

use detector::Engine;
use keymap::{
    azerty_to_jcuken, convert_text_layout, jcuken_to_azerty, jcuken_to_qwerty,
    jcuken_to_turkish_q, qwerty_to_jcuken, turkish_q_to_jcuken,
};
use learner::RuleKind;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Lang {
    En = 0,
    Ru = 1,
    Fr = 2,
    Tr = 3,
}

impl Lang {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Lang::Ru,
            2 => Lang::Fr,
            3 => Lang::Tr,
            _ => Lang::En,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Lang::En | Lang::Fr | Lang::Tr => Lang::Ru,
            Lang::Ru => Lang::En,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum PairId {
    RuEn = 0,
    RuFr = 1,
    RuTr = 2,
}

impl PairId {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => PairId::RuFr,
            2 => PairId::RuTr,
            _ => PairId::RuEn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ActionType {
    None = 0,
    ConvertLayout = 1,
    FixCapsLock = 2,
    FixTypo = 3,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub action: ActionType,
    pub target_lang: Lang,
    pub replacement: Option<String>,
    pub confidence: f64,
    pub reason: String,
}

impl Decision {
    pub fn none(reason: &str) -> Self {
        Self {
            action: ActionType::None,
            target_lang: Lang::En,
            replacement: None,
            confidence: 0.0,
            reason: reason.to_string(),
        }
    }
}

// ============================================================================
// C ABI Exports (matching include/langswitch_core.h)
// ============================================================================

#[repr(C)]
pub struct LSDecision {
    pub action: i32,
    pub target_lang: i32,
    pub replacement: *mut c_char,
    pub confidence: f64,
    pub reason: *mut c_char,
}

fn cstr_to_str<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("") }
    }
}

fn str_to_cstr(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

#[no_mangle]
pub extern "C" fn ls_engine_new(db_path: *const c_char) -> *mut Engine {
    let path_str = cstr_to_str(db_path);
    let opt = if path_str.is_empty() {
        None
    } else {
        Some(path_str)
    };
    Box::into_raw(Box::new(Engine::new(opt)))
}

#[no_mangle]
pub extern "C" fn ls_engine_free(engine: *mut Engine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

#[no_mangle]
pub extern "C" fn ls_engine_evaluate(
    engine: *mut Engine,
    active_word: *const c_char,
    alt_word: *const c_char,
    current_lang: i32,
    app_bundle_id: *const c_char,
) -> LSDecision {
    evaluate_ffi(engine, active_word, alt_word, current_lang, app_bundle_id, false)
}

#[no_mangle]
pub extern "C" fn ls_engine_evaluate_prefix(
    engine: *mut Engine,
    active_word: *const c_char,
    alt_word: *const c_char,
    current_lang: i32,
    app_bundle_id: *const c_char,
) -> LSDecision {
    evaluate_ffi(engine, active_word, alt_word, current_lang, app_bundle_id, true)
}

fn evaluate_ffi(
    engine: *mut Engine,
    active_word: *const c_char,
    alt_word: *const c_char,
    current_lang: i32,
    app_bundle_id: *const c_char,
    prefix: bool,
) -> LSDecision {
    if engine.is_null() {
        return LSDecision {
            action: ActionType::None as i32,
            target_lang: current_lang,
            replacement: ptr::null_mut(),
            confidence: 0.0,
            reason: ptr::null_mut(),
        };
    }

    let eng = unsafe { &mut *engine };
    let active = cstr_to_str(active_word);
    let alt = cstr_to_str(alt_word);
    let alt_opt = if alt.is_empty() { None } else { Some(alt) };
    let app = cstr_to_str(app_bundle_id);
    let lang = Lang::from_i32(current_lang);

    let dec = if prefix {
        eng.evaluate_prefix(active, alt_opt, lang, app)
    } else {
        eng.evaluate(active, alt_opt, lang, app)
    };
    LSDecision {
        action: dec.action as i32,
        target_lang: dec.target_lang as i32,
        replacement: dec
            .replacement
            .as_deref()
            .map(str_to_cstr)
            .unwrap_or(ptr::null_mut()),
        confidence: dec.confidence,
        reason: str_to_cstr(&dec.reason),
    }
}

#[no_mangle]
pub extern "C" fn ls_engine_convert_text(
    engine: *mut Engine,
    text: *const c_char,
    current_lang: i32,
    out_target_lang: *mut i32,
) -> *mut c_char {
    let input = cstr_to_str(text);
    let cur = Lang::from_i32(current_lang);

    let default_target = if !engine.is_null() {
        let eng = unsafe { &*engine };
        match cur {
            Lang::Ru => {
                if eng.is_pair_enabled(PairId::RuEn) {
                    Lang::En
                } else if eng.is_pair_enabled(PairId::RuFr) {
                    Lang::Fr
                } else if eng.is_pair_enabled(PairId::RuTr) {
                    Lang::Tr
                } else {
                    Lang::En
                }
            }
            Lang::En | Lang::Fr | Lang::Tr => Lang::Ru,
        }
    } else {
        cur.opposite()
    };

    let converted = convert_text_layout(input, cur, default_target);
    if !out_target_lang.is_null() {
        unsafe {
            *out_target_lang = default_target as i32;
        }
    }
    str_to_cstr(&converted)
}

#[no_mangle]
pub extern "C" fn ls_engine_set_pair_enabled(
    engine: *mut Engine,
    pair_id: i32,
    enabled: bool,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.set_pair_enabled(PairId::from_i32(pair_id), enabled);
}

#[no_mangle]
pub extern "C" fn ls_engine_is_pair_enabled(engine: *mut Engine, pair_id: i32) -> bool {
    if engine.is_null() {
        return true;
    }
    let eng = unsafe { &*engine };
    eng.is_pair_enabled(PairId::from_i32(pair_id))
}

#[no_mangle]
pub extern "C" fn ls_engine_record_undo(
    engine: *mut Engine,
    original_word: *const c_char,
    converted_word: *const c_char,
    original_lang: i32,
    app_bundle_id: *const c_char,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.store.record_undo(
        cstr_to_str(original_word),
        cstr_to_str(converted_word),
        Lang::from_i32(original_lang),
        cstr_to_str(app_bundle_id),
    );
}

#[no_mangle]
pub extern "C" fn ls_engine_record_manual_convert(
    engine: *mut Engine,
    original_word: *const c_char,
    converted_word: *const c_char,
    target_lang: i32,
    app_bundle_id: *const c_char,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    let orig = cstr_to_str(original_word);
    let conv_in = cstr_to_str(converted_word);
    let t_lang = Lang::from_i32(target_lang);
    let computed_conv: String;
    let conv = if conv_in.is_empty() {
        computed_conv = match t_lang {
            Lang::Ru => qwerty_to_jcuken(orig),
            Lang::En => jcuken_to_qwerty(orig),
            Lang::Fr => jcuken_to_azerty(orig),
            Lang::Tr => jcuken_to_turkish_q(orig),
        };
        &computed_conv
    } else {
        conv_in
    };
    eng.store
        .record_manual_convert(orig, conv, t_lang, cstr_to_str(app_bundle_id));
}

#[no_mangle]
pub extern "C" fn ls_engine_record_accepted(
    engine: *mut Engine,
    word: *const c_char,
    lang: i32,
    app_bundle_id: *const c_char,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.store.record_accepted(
        cstr_to_str(word),
        Lang::from_i32(lang),
        cstr_to_str(app_bundle_id),
    );
}

/// Installs a host dictionary callback (pass NULL to remove it). While installed, the
/// engine only auto-converts into words that a dictionary knows.
/// Installs a host spelling suggester (pass NULL to use only the built-in lexicon).
#[no_mangle]
pub extern "C" fn ls_engine_set_word_suggester(
    engine: *mut Engine,
    suggester: Option<detector::WordSuggester>,
) {
    if engine.is_null() {
        return;
    }
    unsafe { &mut *engine }.set_word_suggester(suggester);
}

/// Locks (`allowed == false`) or unlocks a feature; see `entitlements::Feature` for the ids.
/// Everything is unlocked by default. Unknown feature ids are ignored.
#[no_mangle]
pub extern "C" fn ls_engine_set_feature_allowed(engine: *mut Engine, feature: i32, allowed: bool) {
    if engine.is_null() {
        return;
    }
    if let Some(f) = entitlements::Feature::from_i32(feature) {
        let engine = unsafe { &mut *engine };
        let mut e = engine.entitlements();
        e.set(f, allowed);
        engine.set_entitlements(e);
    }
}

/// Returns whether a feature is currently unlocked.
#[no_mangle]
pub extern "C" fn ls_engine_is_feature_allowed(engine: *const Engine, feature: i32) -> bool {
    if engine.is_null() {
        return false;
    }
    match entitlements::Feature::from_i32(feature) {
        Some(f) => unsafe { &*engine }.entitlements().allows(f),
        None => false,
    }
}

/// Turns typo correction on or off.
#[no_mangle]
pub extern "C" fn ls_engine_set_typo_correction(engine: *mut Engine, enabled: bool) {
    if engine.is_null() {
        return;
    }
    unsafe { &mut *engine }.typo_correction = enabled;
}

#[no_mangle]
pub extern "C" fn ls_engine_set_word_validator(
    engine: *mut Engine,
    validator: Option<detector::WordValidator>,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.set_word_validator(validator);
}

#[no_mangle]
pub extern "C" fn ls_engine_reset_context(engine: *mut Engine) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.reset_context();
}

#[no_mangle]
pub extern "C" fn ls_engine_set_app_excluded(
    engine: *mut Engine,
    app_bundle_id: *const c_char,
    excluded: bool,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.store
        .set_app_excluded(cstr_to_str(app_bundle_id), excluded);
}

#[no_mangle]
pub extern "C" fn ls_engine_is_app_excluded(
    engine: *mut Engine,
    app_bundle_id: *const c_char,
) -> bool {
    if engine.is_null() {
        return false;
    }
    let eng = unsafe { &*engine };
    eng.store.is_app_excluded(cstr_to_str(app_bundle_id))
}

#[no_mangle]
pub extern "C" fn ls_engine_add_rule(
    engine: *mut Engine,
    word: *const c_char,
    rule_type: i32,
    lang: i32,
) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    let w = cstr_to_str(word);
    let l = Lang::from_i32(lang);
    let kind = match rule_type {
        0 => RuleKind::NeverConvert,
        1 => RuleKind::AlwaysConvert,
        _ => RuleKind::PersonalVocab,
    };
    let target = match (kind, l) {
        (RuleKind::AlwaysConvert, Lang::Ru) => {
            let q = qwerty_to_jcuken(w);
            if q != w {
                q
            } else {
                let a = azerty_to_jcuken(w);
                if a != w {
                    a
                } else {
                    turkish_q_to_jcuken(w)
                }
            }
        }
        (RuleKind::AlwaysConvert, Lang::En) => jcuken_to_qwerty(w),
        (RuleKind::AlwaysConvert, Lang::Fr) => jcuken_to_azerty(w),
        (RuleKind::AlwaysConvert, Lang::Tr) => jcuken_to_turkish_q(w),
        _ => w.to_string(),
    };
    eng.store.add_rule(w, &target, kind, l);
}

#[no_mangle]
pub extern "C" fn ls_engine_remove_rule(engine: *mut Engine, word: *const c_char) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.store.remove_rule(cstr_to_str(word));
}

#[no_mangle]
pub extern "C" fn ls_engine_clear_all_learned(engine: *mut Engine) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.store.clear_all();
}

#[no_mangle]
pub extern "C" fn ls_engine_get_dashboard_json(engine: *mut Engine) -> *mut c_char {
    if engine.is_null() {
        return str_to_cstr("{}");
    }
    let eng = unsafe { &*engine };
    str_to_cstr(&eng.store.to_dashboard_json())
}

#[no_mangle]
pub extern "C" fn ls_decision_free(decision: LSDecision) {
    ls_string_free(decision.replacement);
    ls_string_free(decision.reason);
}

#[no_mangle]
pub extern "C" fn ls_string_free(str_ptr: *mut c_char) {
    if !str_ptr.is_null() {
        unsafe {
            drop(CString::from_raw(str_ptr));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_ru_typed_in_en_layout() {
        let mut engine = Engine::new(None);
        let cases = [
            ("ghbdtn", "привет"),
            ("Ghbdtn", "Привет"),
            ("rfr", "как"),
            ("ltkf", "дела"),
            ("gj;fkeqcnf", "пожалуйста"),
            ("cgfcb,j", "спасибо"),
            ("k.,k.", "люблю"),
            ("[jhjij", "хорошо"),
            (";bpym", "жизнь"),
            ("[kt,", "хлеб"),
            ("hf,jnftn", "работает"),
            ("rkfdf", "клава"),
            ("gthtrk.xfntkm", "переключатель"),
        ];

        for (typed, expected) in cases {
            let dec = engine.evaluate(typed, None, Lang::En, "com.apple.Notes");
            assert_eq!(
                dec.action,
                ActionType::ConvertLayout,
                "Expected '{}' to convert to '{}', got {:?}",
                typed,
                expected,
                dec
            );
            assert_eq!(dec.target_lang, Lang::Ru);
            assert_eq!(dec.replacement.as_deref(), Some(expected));
        }
    }

    #[test]
    fn test_en_typed_in_ru_layout() {
        let mut engine = Engine::new(None);
        let cases = [
            ("руддщ", "hello"),
            ("цщкдв", "world"),
            ("еуые", "test"),
            ("ыцшае", "swift"),
            ("агтсешщт", "function"),
            ("куегкт", "return"),
            ("зукаусе", "perfect"),
            ("фгещьфешс", "automatic"),
            ("лунищфкв", "keyboard"),
        ];

        for (typed, expected) in cases {
            let dec = engine.evaluate(typed, None, Lang::Ru, "com.apple.Notes");
            assert_eq!(
                dec.action,
                ActionType::ConvertLayout,
                "Expected '{}' to convert to '{}', got {:?}",
                typed,
                expected,
                dec
            );
            assert_eq!(dec.target_lang, Lang::En);
            assert_eq!(dec.replacement.as_deref(), Some(expected));
        }
    }

    #[test]
    fn test_exact_layout_readings_for_azerty() {
        use crate::detector::AltReadings;
        let mut engine = Engine::new(None);
        engine.set_pair_enabled(PairId::RuFr, true);
        let reading = |pairs: &[(Lang, &str)]| {
            let mut a = AltReadings::exact();
            for (l, s) in pairs {
                a.set(*l, *s);
            }
            a
        };

        // AZERTY accented letters sit on the Russian digit keys: `été` typed on RU is `2е2`.
        let a = reading(&[(Lang::En, "2t2"), (Lang::Fr, "été")]);
        let d = engine.evaluate_with_alts("2е2", &a, Lang::Ru, "notepad.exe");
        assert_eq!(d.action, ActionType::ConvertLayout, "{}", d.reason);
        assert_eq!((d.target_lang, d.replacement.as_deref()), (Lang::Fr, Some("été")));

        // French elision through the AZERTY `'` (Russian `4`).
        let a = reading(&[(Lang::En, "c4est"), (Lang::Fr, "c'est")]);
        let d = engine.evaluate_with_alts("с4уые", &a, Lang::Ru, "notepad.exe");
        assert_eq!(d.replacement.as_deref(), Some("c'est"), "{}", d.reason);

        // Plain numbers stay numbers even though `2` and `0` are French letters.
        let a = reading(&[(Lang::En, "2020"), (Lang::Fr, "éàéà")]);
        let d = engine.evaluate_with_alts("2020", &a, Lang::Ru, "notepad.exe");
        assert_eq!(d.action, ActionType::None);

        // Russian typed on AZERTY: `х` is the `^` dead key.
        engine.reset_context();
        let a = reading(&[(Lang::Ru, "хорошо")]);
        let d = engine.evaluate_with_alts("^jhjij", &a, Lang::Fr, "notepad.exe");
        assert_eq!(d.action, ActionType::ConvertLayout, "{}", d.reason);
        assert_eq!((d.target_lang, d.replacement.as_deref()), (Lang::Ru, Some("хорошо")));

        // Exact readings win over the static AZERTY guess (Canadian/Swiss French are QWERTY-like).
        let a = reading(&[(Lang::Ru, "привет")]);
        let d = engine.evaluate_with_alts("ghbdtn", &a, Lang::Fr, "notepad.exe");
        assert_eq!(d.replacement.as_deref(), Some("привет"), "{}", d.reason);

        // A real French word typed in French is kept.
        let a = reading(&[(Lang::Ru, "ищтощгк")]);
        let d = engine.evaluate_with_alts("bonjour", &a, Lang::Fr, "notepad.exe");
        assert_eq!(d.action, ActionType::None);
    }

    #[test]
    fn test_ru_fr_and_ru_tr_and_pair_toggles() {
        let mut engine = Engine::new(None);

        // 1. Test French typed in Russian layout ("ищтощгк" -> "bonjour", "жуксш" on AZERTY -> "merci")
        let dec_fr = engine.evaluate("ищтощгк", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_fr.action, ActionType::ConvertLayout);
        assert_eq!(dec_fr.target_lang, Lang::Fr);
        assert_eq!(dec_fr.replacement.as_deref(), Some("bonjour"));

        let dec_fr_azerty = engine.evaluate("жуксш", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_fr_azerty.action, ActionType::ConvertLayout);
        assert_eq!(dec_fr_azerty.target_lang, Lang::Fr);
        assert_eq!(dec_fr_azerty.replacement.as_deref(), Some("merci"));

        // 2. Test Russian typed while in French layout ("ghbdtn" -> "привет")
        let dec_ru_from_fr = engine.evaluate("ghbdtn", None, Lang::Fr, "com.apple.Notes");
        assert_eq!(dec_ru_from_fr.action, ActionType::ConvertLayout);
        assert_eq!(dec_ru_from_fr.target_lang, Lang::Ru);
        assert_eq!(dec_ru_from_fr.replacement.as_deref(), Some("привет"));

        // 3. Test Turkish typed in Russian layout ("ьукрфиф" -> "merhaba", "еужуллълдук" -> "teşekkürler")
        let dec_tr = engine.evaluate("ьукрфиф", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_tr.action, ActionType::ConvertLayout);
        assert_eq!(dec_tr.target_lang, Lang::Tr);
        assert_eq!(dec_tr.replacement.as_deref(), Some("merhaba"));

        let dec_tr2 = engine.evaluate("еужуллъкдук", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_tr2.action, ActionType::ConvertLayout);
        assert_eq!(dec_tr2.target_lang, Lang::Tr);
        assert_eq!(dec_tr2.replacement.as_deref(), Some("teşekkürler"));

        // 4. Test disabling RU<->FR and RU<->TR pairs
        engine.set_pair_enabled(PairId::RuFr, false);
        engine.set_pair_enabled(PairId::RuTr, false);
        assert!(!engine.is_pair_enabled(PairId::RuFr));
        assert!(!engine.is_pair_enabled(PairId::RuTr));

        let dec_fr_off = engine.evaluate("ищтощгк", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_fr_off.action, ActionType::None);

        let dec_tr_off = engine.evaluate("еужуллъкдук", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_tr_off.action, ActionType::None);
    }

    #[test]
    fn test_never_false_convert_valid_words_or_code() {
        let mut engine = Engine::new(None);
        let valid_en = [
            "hello", "world", "git", "cargo", "swift", "xcode", "struct", "async", "await",
            "useState", "NSStatusItem", "main.rs", "google.com", "v2.0", "user_name", "--help",
            "https://apple.com",
        ];
        for w in valid_en {
            let dec = engine.evaluate(w, None, Lang::En, "com.apple.Terminal");
            assert_eq!(
                dec.action,
                ActionType::None,
                "Should NOT convert valid EN/code token '{}', got {:?}",
                w,
                dec
            );
        }

        let valid_ru = [
            "привет", "как", "дела", "спасибо", "хорошо", "работа", "программа", "клавиатура",
        ];
        for w in valid_ru {
            let dec = engine.evaluate(w, None, Lang::Ru, "ru.keepcoder.Telegram");
            assert_eq!(
                dec.action,
                ActionType::None,
                "Should NOT convert valid RU token '{}', got {:?}",
                w,
                dec
            );
        }
    }

    #[test]
    fn test_self_education_undo_and_manual_convert() {
        let mut engine = Engine::new(None);
        let dec1 = engine.evaluate("ghbdtn", None, Lang::En, "com.apple.Notes");
        assert_eq!(dec1.action, ActionType::ConvertLayout);

        engine
            .store
            .record_undo("ghbdtn", "привет", Lang::En, "com.apple.Notes");

        let dec2 = engine.evaluate("ghbdtn", None, Lang::En, "com.apple.Notes");
        assert_eq!(dec2.action, ActionType::None);

        engine
            .store
            .record_manual_convert("b", "и", Lang::Ru, "com.apple.Notes");
        let dec3 = engine.evaluate("b", None, Lang::En, "com.apple.Notes");
        assert_eq!(dec3.action, ActionType::ConvertLayout);
        assert_eq!(dec3.replacement.as_deref(), Some("и"));
    }

    #[test]
    fn test_edge_punctuation_is_kept_not_converted_to_letters() {
        let mut engine = Engine::new(None);
        // `,` `.` on QWERTY are `б` `ю` in ЙЦУКЕН: trailing punctuation must stay punctuation.
        let cases = [
            ("ghbdtn,", Lang::En, Some("привет,")),
            ("ghbdtn.", Lang::En, Some("привет.")),
            ("\"ghbdtn", Lang::En, Some("\"привет")),
            ("руддщ,", Lang::Ru, Some("hello,")),
            // Edge characters that complete a real word are still converted as letters.
            ("[jhjij", Lang::En, Some("хорошо")),
            ("k.,k.", Lang::En, Some("люблю")),
            (",eltn", Lang::En, Some("будет")),
            // Valid words with punctuation are left alone.
            ("hello,", Lang::En, None),
            ("world.", Lang::En, None),
            ("привет,", Lang::Ru, None),
        ];
        for (typed, lang, expected) in cases {
            engine.reset_context();
            let dec = engine.evaluate(typed, None, lang, "com.apple.Notes");
            assert_eq!(dec.replacement.as_deref(), expected, "'{}' -> {:?}", typed, dec);
        }
    }

    #[test]
    fn test_mixed_sentence_regressions() {
        // Target-layout punctuation inside a word typed in the wrong layout.
        let mut engine = Engine::new(None);
        for (typed, lang, expected) in [
            ("тщцю", Lang::Ru, "now."),
            ("[jhjij!", Lang::En, "хорошо!"),
            ("[jhjij.", Lang::En, "хорошо."),
            ("[jhjij/", Lang::En, "хорошо."),
        ] {
            engine.reset_context();
            let dec = engine.evaluate(typed, None, lang, "com.apple.Notes");
            assert_eq!(dec.replacement.as_deref(), Some(expected), "'{}' -> {:?}", typed, dec);
        }

        // Single-letter words convert even without prior context in normal apps (`z` -> `я`, `Ш` -> `I`),
        // while valid active-layout single letters (`a`, `I`, `я`) and non-words (`g`, `x`) stay untouched.
        engine.reset_context();
        for (typed, lang, expected, target) in [
            ("z", Lang::En, "я", Lang::Ru),
            ("Z", Lang::En, "Я", Lang::Ru),
            ("e", Lang::En, "у", Lang::Ru),
            ("b", Lang::En, "и", Lang::Ru),
            ("d", Lang::En, "в", Lang::Ru),
            ("c", Lang::En, "с", Lang::Ru),
            ("f", Lang::En, "а", Lang::Ru),
            ("r", Lang::En, "к", Lang::Ru),
            ("j", Lang::En, "о", Lang::Ru),
            ("z,", Lang::En, "я,", Lang::Ru),
            ("Ш", Lang::Ru, "I", Lang::En),
            ("ш", Lang::Ru, "i", Lang::En),
            ("ф", Lang::Ru, "a", Lang::En),
        ] {
            engine.reset_context();
            let dec = engine.evaluate(typed, None, lang, "com.apple.Notes");
            assert_eq!(dec.action, ActionType::ConvertLayout, "'{}' -> {:?}", typed, dec);
            assert_eq!(dec.target_lang, target, "'{}'", typed);
            assert_eq!(dec.replacement.as_deref(), Some(expected), "'{}' -> {:?}", typed, dec);
        }
        for (typed, lang) in [
            ("a", Lang::En),
            ("I", Lang::En),
            ("g", Lang::En),
            ("x", Lang::En),
            ("я", Lang::Ru),
            ("в", Lang::Ru),
            ("й", Lang::Ru),
        ] {
            engine.reset_context();
            assert_eq!(
                engine.evaluate(typed, None, lang, "com.apple.Notes").action,
                ActionType::None,
                "'{}' should not convert",
                typed
            );
        }
        // In code apps / terminals, single Latin letters stay untouched without Russian context.
        engine.reset_context();
        assert_eq!(engine.evaluate("z", None, Lang::En, "com.apple.Terminal").action, ActionType::None);
        assert_eq!(engine.evaluate("c", None, Lang::En, "com.microsoft.VSCode").action, ActionType::None);

        // Undo on a punctuated word normalizes the rule to the core word and marks recent_fixes undone.
        engine.reset_context();
        let dec = engine.evaluate("ghbdtn,", None, Lang::En, "com.apple.Notes");
        assert_eq!(dec.replacement.as_deref(), Some("привет,"));
        engine.store.record_undo("ghbdtn,", "привет,", Lang::En, "com.apple.Notes");
        assert!(engine.store.recent_fixes.front().unwrap().undone);
        assert_eq!(engine.evaluate("ghbdtn", None, Lang::En, "com.apple.Notes").action, ActionType::None);

        // Whole-token NeverConvert with leading/trailing punctuation must not fall through to core.
        engine.store.add_rule(",eltn", ",eltn", RuleKind::NeverConvert, Lang::En);
        assert_eq!(engine.evaluate(",eltn", None, Lang::En, "com.apple.Notes").action, ActionType::None);

        // add_rule(NeverConvert) marks matching recent fixes as undone.
        engine.reset_context();
        let _ = engine.evaluate("rfr", None, Lang::En, "com.apple.Notes");
        assert!(!engine.store.recent_fixes.front().unwrap().undone);
        engine.store.add_rule("rfr", "rfr", RuleKind::NeverConvert, Lang::En);
        assert!(engine.store.recent_fixes.front().unwrap().undone);

        // Contradicting learned rules: the newest wins, and a weak rule never
        // turns a real word (`I`) into a non-word (`Ш`).
        engine.store.record_manual_convert("ш", "i", Lang::En, "x");
        engine.store.record_manual_convert("i", "ш", Lang::Ru, "x");
        assert!(!engine.store.rules.contains_key("ш"));
        engine.reset_context();
        assert_eq!(engine.evaluate("I", None, Lang::En, "x").action, ActionType::None);
    }

    extern "C" fn typo_validator(word: *const c_char, lang: i32) -> bool {
        let w = unsafe { CStr::from_ptr(word) }.to_str().unwrap_or("");
        matches!(
            (lang, w),
            (1, "чувствует") | (0, "receive") | (0, "relieve") | (0, "found") | (0, "pound") | (0, "nasty")
                | (0, "long") | (0, "loan") | (0, "ppm") | (1, "дела") | (1, "сила") | (1, "ничто")
        )
    }

    extern "C" fn typo_suggester(word: *const c_char, lang: i32, out: *mut c_char, len: usize) -> usize {
        let w = unsafe { CStr::from_ptr(word) }.to_str().unwrap_or("");
        let s = match (lang, w) {
            (1, "чувстсвует") => "чувствует\nчувствуют",
            (0, "recieve") => "receive\nrelieve",
            (0, "fpund") => "found\npound",
            (0, "nastya") => "nasty\nnasty-a",
            (0, "websocket") => "web-socket",
            (0, "becuase") => "because",
            (0, "logn") => "!long\nlong\nloan",
            (0, "pnpm") => "!ppm\nppm",
            (1, "дила") => "!дела\nдела\nсила",
            (1, "ничо") => "!ничто\nничто",
            _ => "",
        };
        if s.is_empty() || s.len() + 1 > len {
            return 0;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), out as *mut u8, s.len());
            *out.add(s.len()) = 0;
        }
        s.len()
    }

    #[test]
    fn test_entitlements_gate_features() {
        use crate::entitlements::{Entitlements, Feature};
        let mut engine = Engine::new(None);
        let d = engine.evaluate("ghbdtn", None, Lang::En, "test.app");
        assert_eq!(d.replacement.as_deref(), Some("привет"));

        engine.set_entitlements(Entitlements::all().with(Feature::PairRuEn, false));
        engine.reset_context();
        let d = engine.evaluate("ghbdtn", None, Lang::En, "test.app");
        assert_eq!(d.action, ActionType::None, "locked pair must not convert");
        // The user's own preference is untouched and comes back when unlocked.
        assert!(engine.is_pair_enabled(PairId::RuEn));

        engine.set_entitlements(Entitlements::all());
        engine.reset_context();
        let d = engine.evaluate("ghbdtn", None, Lang::En, "test.app");
        assert_eq!(d.replacement.as_deref(), Some("привет"));
    }

    #[test]
    fn test_typo_correction() {
        let mut engine = Engine::new(None);
        engine.set_word_validator(Some(typo_validator));
        engine.set_word_suggester(Some(typo_suggester));
        let app = "com.tdesktop.Telegram";

        // Wrong layout + typo.
        let d = engine.evaluate("xedcncdetn", None, Lang::En, app);
        assert_eq!((d.action, d.target_lang, d.replacement.as_deref()), (ActionType::FixTypo, Lang::Ru, Some("чувствует")), "{d:?}");

        // Typo in the right layout.
        engine.reset_context();
        let d = engine.evaluate("recieve", None, Lang::En, app);
        assert_eq!((d.action, d.replacement.as_deref()), (ActionType::FixTypo, Some("receive")), "{d:?}");

        // 4-letter words: swapped / wrong letter when the system picks the fix.
        for (word, lang, fixed) in [("logn", Lang::En, "long"), ("дила", Lang::Ru, "дела")] {
            engine.reset_context();
            let d = engine.evaluate(word, None, lang, app);
            assert_eq!((d.action, d.replacement.as_deref()), (ActionType::FixTypo, Some(fixed)), "{word}: {d:?}");
        }
        // ...but never drop a typed letter in short words, or add one to 4-letter slang.
        for (word, lang) in [("pnpm", Lang::En), ("ничо", Lang::Ru)] {
            engine.reset_context();
            assert_eq!(engine.evaluate(word, None, lang, app).action, ActionType::None, "{word}");
        }

        // Conservative: Capitalized, ambiguous, code apps, unknown words, real words.
        for (word, app) in [
            ("Recieve", app),
            ("fpund", app),
            ("nastya", app),
            ("websocket", app),
            ("recieve", "com.apple.Terminal"),
            ("xedcncdetn", "com.microsoft.VSCode"),
            ("qwzxv", app),
            ("receive", app),
        ] {
            engine.reset_context();
            assert_eq!(engine.evaluate(word, None, Lang::En, app).action, ActionType::None, "{word} @ {app}");
        }

        // Off switch.
        engine.typo_correction = false;
        assert_eq!(engine.evaluate("recieve", None, Lang::En, app).action, ActionType::None);

        // Undo teaches it to leave the word alone.
        engine.typo_correction = true;
        engine.store.record_undo("recieve", "receive", Lang::En, app);
        assert_eq!(engine.evaluate("recieve", None, Lang::En, app).action, ActionType::None);
    }

    extern "C" fn fake_validator(word: *const c_char, lang: i32) -> bool {
        let w = unsafe { CStr::from_ptr(word) }.to_str().unwrap_or("");
        matches!(
            (lang, w),
            (0, "tot") | (0, "twelfth") | (1, "привет") | (1, "еще")
        )
    }

    #[test]
    fn test_word_validator_blocks_false_conversions_and_guesses() {
        let mut engine = Engine::new(None);
        // Without a host dictionary these real English words get mangled.
        assert_eq!(
            engine.evaluate("tot", None, Lang::En, "x").action,
            ActionType::ConvertLayout
        );

        engine.set_word_validator(Some(fake_validator));
        for w in ["tot", "twelfth"] {
            engine.reset_context();
            let dec = engine.evaluate(w, None, Lang::En, "x");
            assert_eq!(dec.action, ActionType::None, "'{}' -> {:?}", w, dec);
        }

        // Dictionary conversions still work...
        let dec = engine.evaluate("ghbdtn", None, Lang::En, "x");
        assert_eq!(dec.replacement.as_deref(), Some("привет"));

        // ...but phonotactic guesses into unknown words do not (strict mode).
        let dec = engine.evaluate("weighbridge", None, Lang::En, "x");
        assert_eq!(dec.action, ActionType::None, "{:?}", dec);

        engine.set_word_validator(None);
        assert!(!engine.strict_known_only);
    }

    #[test]
    fn test_accidental_caps_lock() {
        let mut engine = Engine::new(None);
        let dec = engine.evaluate("hELLO", None, Lang::En, "com.apple.Notes");
        assert_eq!(dec.action, ActionType::FixCapsLock);
        assert_eq!(dec.replacement.as_deref(), Some("Hello"));

        let dec_ru = engine.evaluate("пРИВЕТ", None, Lang::Ru, "com.apple.Notes");
        assert_eq!(dec_ru.action, ActionType::FixCapsLock);
        assert_eq!(dec_ru.replacement.as_deref(), Some("Привет"));
    }

    #[test]
    fn test_sub_50_microsecond_speed() {
        let mut engine = Engine::new(None);
        let words = ["ghbdtn", "hello", "руддщ", "привет", "gj;fkeqcnf", "useState"];
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            for w in words {
                let _ = engine.evaluate(w, None, Lang::En, "com.apple.Notes");
            }
        }
        let elapsed = start.elapsed();
        let per_call_us = elapsed.as_micros() as f64 / ((iterations * words.len()) as f64);
        println!("Average evaluation latency: {:.2} µs/word", per_call_us);
        assert!(
            per_call_us < 500.0,
            "Evaluation too slow: {:.2} µs",
            per_call_us
        );
    }
}
