use langswitch_core::detector::{AltReadings, Engine};
use langswitch_core::entitlements::{Entitlements, Feature};
use langswitch_core::learner::RuleKind;
use langswitch_core::lexicon::lexicon;
use langswitch_core::{ActionType, Lang, PairId};
use std::ffi::CString;

const APP: &str = "notes.app";

#[test]
fn converts_confident_prefixes_before_word_boundaries() {
    for (typed, lang, expected, target) in [
        ("ghbd", Lang::En, "прив", Lang::Ru),
        ("Ghbd", Lang::En, "Прив", Lang::Ru),
        ("GHBD", Lang::En, "ПРИВ", Lang::Ru),
        ("рудд", Lang::Ru, "hell", Lang::En),
        ("ьукр", Lang::Ru, "merh", Lang::Tr),
    ] {
        let mut engine = Engine::new(None);
        let d = engine.evaluate_prefix(typed, None, lang, APP);
        assert_eq!(d.action, ActionType::ConvertLayout, "{typed}: {d:?}");
        assert_eq!(d.replacement.as_deref(), Some(expected));
        assert_eq!(d.target_lang, target);
        assert!(
            engine.store.word_frequency.is_empty(),
            "fragments must not become vocabulary"
        );
        assert!(
            engine.store.apps.is_empty(),
            "fragments must not update app priors"
        );
        assert_eq!(engine.last_lang, Some(target));
        assert_eq!(engine.store.total_auto_conversions, 1);
    }
}

#[test]
fn preserves_every_builtin_word_while_it_is_being_typed() {
    let mut engine = Engine::new(None);
    for lang in [Lang::En, Lang::Ru, Lang::Fr, Lang::Tr] {
        for word in lexicon(lang) {
            let mut prefix = String::new();
            for c in word.chars() {
                prefix.push(c);
                let d = engine.evaluate_prefix(&prefix, None, lang, APP);
                assert_eq!(d.action, ActionType::None, "{word} at {prefix}: {d:?}");
            }
        }
    }
    assert_eq!(engine.store.total_auto_conversions, 0);
    assert!(engine.store.rules.is_empty());
    assert!(engine.store.apps.is_empty());
    assert!(engine.store.word_frequency.is_empty());
    assert_eq!(engine.last_lang, None);
}

#[test]
fn waits_on_short_guarded_or_plausible_fragments() {
    let mut engine = Engine::new(None);
    for word in [
        "g",
        "gh",
        "ghb",
        "plau",
        "hELLO",
        "gHBD",
        "ghbd123",
        "ghbd_word",
        "ghBd",
        "ghbd/path",
        "https://ghbd",
        "ghbd.rs",
        "ghbd ",
        " ghbd",
    ] {
        assert_eq!(
            engine.evaluate_prefix(word, None, Lang::En, APP).action,
            ActionType::None,
            "{word}"
        );
    }
    assert_eq!(
        engine
            .evaluate_prefix("ghbd", None, Lang::En, "com.apple.Terminal")
            .action,
        ActionType::None
    );
    engine.store.set_app_excluded(APP, true);
    assert_eq!(
        engine.evaluate_prefix("ghbd", None, Lang::En, APP).action,
        ActionType::None
    );
    assert_eq!(engine.store.total_auto_conversions, 0);
}

#[test]
fn respects_pairs_entitlements_and_undo_for_whole_words_and_prefixes() {
    let mut engine = Engine::new(None);
    engine.set_pair_enabled(PairId::RuEn, false);
    assert_eq!(
        engine.evaluate_prefix("ghbd", None, Lang::En, APP).action,
        ActionType::None
    );
    engine.set_pair_enabled(PairId::RuEn, true);
    engine.set_entitlements(Entitlements::all().with(Feature::PairRuEn, false));
    assert_eq!(
        engine.evaluate_prefix("ghbd", None, Lang::En, APP).action,
        ActionType::None
    );
    engine.set_entitlements(Entitlements::all());
    engine.store.record_undo("ghbdtn", "привет", Lang::En, APP);
    assert_eq!(
        engine.evaluate_prefix("ghbd", None, Lang::En, APP).action,
        ActionType::None
    );
    engine.store.remove_rule("ghbdtn");
    engine.store.record_undo("ghbd", "прив", Lang::En, APP);
    assert_eq!(
        engine.evaluate_prefix("ghbdt", None, Lang::En, APP).action,
        ActionType::None
    );
    engine.store.remove_rule("ghbd");
    // A short 1-3 letter NeverConvert rule (e.g. from undoing a single-letter word)
    // must NOT disable early prefix conversion for unrelated longer words starting with it.
    engine.store.record_undo("g", "п", Lang::En, APP);
    assert_eq!(
        engine.evaluate_prefix("ghbd", None, Lang::En, APP).action,
        ActionType::ConvertLayout
    );
    engine.store.remove_rule("g");
    engine.store.add_rule(
        "ghbdcustom",
        "ghbdcustom",
        RuleKind::PersonalVocab,
        Lang::En,
    );
    assert_eq!(
        engine.evaluate_prefix("ghbd", None, Lang::En, APP).action,
        ActionType::None
    );
}

#[test]
fn uses_exact_layout_readings_and_waits_when_targets_are_ambiguous() {
    let mut engine = Engine::new(None);
    let mut alts = AltReadings::exact();
    alts.set(Lang::En, "hell");
    alts.set(Lang::Fr, "bonj");
    alts.set(Lang::Tr, "merh");
    assert_eq!(
        engine
            .evaluate_prefix_with_alts("рудд", &alts, Lang::Ru, APP)
            .action,
        ActionType::None
    );
    engine.set_pair_enabled(PairId::RuEn, false);
    engine.set_pair_enabled(PairId::RuTr, false);
    let d = engine.evaluate_prefix_with_alts("рудд", &alts, Lang::Ru, APP);
    assert_eq!(d.target_lang, Lang::Fr);
    assert_eq!(d.replacement.as_deref(), Some("bonj"));

    // No static fallback may invent a target the host cannot read/switch to.
    let mut engine = Engine::new(None);
    let missing = AltReadings::exact();
    assert_eq!(
        engine
            .evaluate_prefix_with_alts("ghbd", &missing, Lang::En, APP)
            .action,
        ActionType::None
    );
}

#[test]
fn c_abi_supports_prefix_evaluation_and_null_engine() {
    use langswitch_core::{
        ls_decision_free, ls_engine_evaluate_prefix, ls_engine_free, ls_engine_new,
    };
    let word = CString::new("ghbd").unwrap();
    let app = CString::new(APP).unwrap();
    let engine = ls_engine_new(std::ptr::null());
    let d = ls_engine_evaluate_prefix(
        engine,
        word.as_ptr(),
        std::ptr::null(),
        Lang::En as i32,
        app.as_ptr(),
    );
    assert_eq!(d.action, ActionType::ConvertLayout as i32);
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(d.replacement) }
            .to_str()
            .unwrap(),
        "прив"
    );
    ls_decision_free(d);
    ls_engine_free(engine);
    let d = ls_engine_evaluate_prefix(
        std::ptr::null_mut(),
        word.as_ptr(),
        std::ptr::null(),
        0,
        app.as_ptr(),
    );
    assert_eq!(d.action, ActionType::None as i32);
    ls_decision_free(d);
}
