use crate::keymap::{
    azerty_to_jcuken, detect_inverted_caps, jcuken_to_azerty, jcuken_to_qwerty,
    jcuken_to_turkish_q, qwerty_to_jcuken, turkish_q_to_jcuken,
};
use crate::learner::{LearningStore, RuleKind};
use crate::lexicon::{
    is_known_en_word, is_known_fr_word, is_known_ru_word, is_known_tr_word,
    score_en_phonotactics, score_fr_phonotactics, score_ru_phonotactics, score_tr_phonotactics,
};
use crate::typo::{
    builtin_suggestions, edit_distance, edit_kind, is_adjacent_swap, is_code_app,
    max_cross_layout_edits, EditKind, MIN_SAME_LAYOUT_LEN, MIN_UNBACKED_EDIT_LEN,
};
use crate::entitlements::{Entitlements, Feature};
use crate::{ActionType, Decision, Lang, PairId};
use std::ffi::CString;
use std::os::raw::c_char;

/// Optional host-provided dictionary (e.g. the macOS system spell checker).
/// Receives a lowercase UTF-8 word and an `LSLang` value; returns `true` if the
/// word is a correctly spelled word in that language.
pub type WordValidator = extern "C" fn(word: *const c_char, lang: i32) -> bool;

/// Optional host spelling suggester (e.g. `NSSpellChecker` guesses). Receives a lowercase
/// UTF-8 word and an `LSLang`; writes newline-separated suggestions, best first, as a
/// NUL-terminated UTF-8 string into `out` (capacity `out_len`) and returns the number of
/// bytes written (0 = no suggestions).
pub type WordSuggester =
    extern "C" fn(word: *const c_char, lang: i32, out: *mut c_char, out_len: usize) -> usize;

/// How a token should be evaluated (internal).
#[derive(Clone, Copy)]
struct EvalOpts {
    /// Only convert into words that are known (dictionary / validator / personal vocab).
    require_known: bool,
    /// Apply learning side effects (context, accepted-word stats) when nothing is converted.
    commit_none: bool,
    /// The code/number guardrail was already cleared by an alternate layout reading
    /// (e.g. `2е2` typed on Russian is `été` on French AZERTY).
    skip_guard: bool,
}

/// The same physical keystrokes read through other keyboard layouts, by language.
///
/// Hosts that can translate keys through the real installed layouts (Windows `ToUnicodeEx`)
/// pass one *exact* reading per target language; the engine then trusts those readings and
/// skips its static keymap guesses. The legacy single `alt` string maps onto this as well.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AltReadings {
    slots: [Option<String>; 4],
    exact: bool,
}

impl AltReadings {
    /// Readings that come straight from the installed layouts: static keymaps are skipped
    /// for every language that has one.
    pub fn exact() -> Self {
        Self { slots: Default::default(), exact: true }
    }

    pub fn set(&mut self, lang: Lang, reading: impl Into<String>) {
        let r = reading.into();
        self.slots[lang as usize] = (!r.is_empty()).then_some(r);
    }

    pub fn get(&self, lang: Lang) -> Option<&str> {
        self.slots[lang as usize].as_deref()
    }

    /// The legacy single alternate reading: the RU view for a Latin layout, the EN view for RU.
    pub fn from_legacy(alt: Option<&str>, current: Lang) -> Self {
        let mut a = Self::default();
        if let Some(s) = alt {
            a.set(if current == Lang::Ru { Lang::En } else { Lang::Ru }, s);
        }
        a
    }

    /// Characters `start..end` of each reading that is aligned with a `total`-char word.
    fn slice(&self, total: usize, start: usize, end: usize) -> Self {
        let mut out = Self { slots: Default::default(), exact: self.exact };
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(s) = slot {
                let chars: Vec<char> = s.chars().collect();
                if chars.len() == total {
                    out.slots[i] = Some(chars[start..end].iter().collect());
                }
            }
        }
        out
    }

    /// The reading for `lang` unless it is identical to what was typed.
    fn usable(&self, lang: Lang, typed: &str) -> Option<&str> {
        self.get(lang).filter(|s| *s != typed)
    }
}

pub struct Engine {
    pub store: LearningStore,
    pub last_lang: Option<Lang>,
    validator: Option<WordValidator>,
    /// When `true`, never convert into a word that no dictionary knows
    /// (no "phonotactic guesses"). Enabled automatically when a validator is installed.
    pub strict_known_only: bool,
    suggester: Option<WordSuggester>,
    /// Fix misspelled words (and wrong-layout words that also contain a typo).
    pub typo_correction: bool,
    /// Which features the host has unlocked (everything by default).
    entitlements: Entitlements,
}

impl Engine {
    pub fn new(db_path: Option<&str>) -> Self {
        Self {
            store: LearningStore::open(db_path),
            last_lang: None,
            validator: None,
            strict_known_only: false,
            suggester: None,
            typo_correction: true,
            entitlements: Entitlements::all(),
        }
    }

    /// Installs (or removes) a host dictionary. Installing one also turns on
    /// `strict_known_only`, so auto-conversions only ever produce real words.
    pub fn set_word_validator(&mut self, validator: Option<WordValidator>) {
        self.validator = validator;
        self.strict_known_only = validator.is_some();
    }

    pub fn set_word_suggester(&mut self, suggester: Option<WordSuggester>) {
        self.suggester = suggester;
    }

    /// Restricts or unlocks features. Locked features act as if switched off; the user's own
    /// preferences (e.g. persisted pair toggles) are left untouched.
    pub fn set_entitlements(&mut self, entitlements: Entitlements) {
        self.entitlements = entitlements;
    }

    pub fn entitlements(&self) -> Entitlements {
        self.entitlements
    }

    /// The user wants the pair on *and* the host has unlocked it.
    fn pair_active(&self, pair: PairId) -> bool {
        let feature = match pair {
            PairId::RuEn => Feature::PairRuEn,
            PairId::RuFr => Feature::PairRuFr,
            PairId::RuTr => Feature::PairRuTr,
        };
        self.store.is_pair_enabled(pair) && self.entitlements.allows(feature)
    }

    pub fn reset_context(&mut self) {
        self.last_lang = None;
    }

    pub fn set_pair_enabled(&mut self, pair: PairId, enabled: bool) {
        self.store.set_pair_enabled(pair, enabled);
    }

    pub fn is_pair_enabled(&self, pair: PairId) -> bool {
        self.store.is_pair_enabled(pair)
    }

    pub fn evaluate(
        &mut self,
        active_word: &str,
        alt_word_opt: Option<&str>,
        current_lang: Lang,
        app_bundle_id: &str,
    ) -> Decision {
        let alts = AltReadings::from_legacy(alt_word_opt, current_lang);
        self.evaluate_with_alts(active_word, &alts, current_lang, app_bundle_id)
    }

    /// Like [`Engine::evaluate`], with one reading of the keystrokes per target layout.
    pub fn evaluate_with_alts(
        &mut self,
        active_word: &str,
        alts: &AltReadings,
        current_lang: Lang,
        app_bundle_id: &str,
    ) -> Decision {
        if !app_bundle_id.is_empty() && self.store.is_app_excluded(app_bundle_id) {
            return Decision::none("App excluded");
        }

        let trimmed = active_word.trim_matches(|c: char| c.is_whitespace());
        if trimmed.is_empty() {
            return Decision::none("Empty word");
        }

        // Digits and symbols usually mean code or numbers. But keys that are symbols in one
        // layout can be letters in another (AZERTY `é` `è` `ç` `à` `'` sit on the Russian
        // digit keys; Russian `х` is the French `^` dead key). Such a token is only converted
        // into a real word, and only through an exact reading from the installed layout.
        let mut skip_guard = false;
        if should_bypass_token(trimmed, current_lang) {
            if !self.alt_clears_guard(trimmed, alts, current_lang) {
                return Decision::none("Guardrail: code/number/URL/path");
            }
            skip_guard = true;
        }
        let opts = |require_known: bool, commit_none: bool| EvalOpts {
            require_known: require_known || skip_guard,
            commit_none,
            skip_guard,
        };

        // Split off leading/trailing characters that are not letters in the active layout.
        // On a QWERTY keyboard `,` `.` `;` `'` `[` `]` are Russian letters (б ю ж э х ъ), so a
        // token like `ghbdtn,` must become `привет,` (comma kept), not `приветб`.
        let chars: Vec<char> = trimmed.chars().collect();
        let first_letter = chars.iter().position(|c| c.is_alphabetic());
        let last_letter = chars.iter().rposition(|c| c.is_alphabetic());

        let (start, end) = match (first_letter, last_letter) {
            (Some(s), Some(e)) => (s, e + 1),
            // No letters at all in the active layout (e.g. `,.`): only a whole-token
            // dictionary match may convert it.
            _ => {
                return self.evaluate_token(
                    trimmed,
                    alts,
                    current_lang,
                    app_bundle_id,
                    opts(true, true),
                );
            }
        };

        if start == 0 && end == chars.len() {
            return self.evaluate_token(
                trimmed,
                alts,
                current_lang,
                app_bundle_id,
                opts(false, true),
            );
        }

        // 1. Whole token, edge characters included (`[jhjij` -> `хорошо`, `k.,k.` -> `люблю`),
        //    but only when the result is a real word.
        let whole = self.evaluate_token(
            trimmed,
            alts,
            current_lang,
            app_bundle_id,
            opts(true, false),
        );
        if whole.action != ActionType::None {
            return whole;
        }

        // 1b. Leading characters kept as letters, trailing punctuation stripped
        //     (`[jhjij.` -> `хорошо.`): `[` is `х`, but `.` is a real full stop.
        if start > 0 && end < chars.len() {
            let head: String = chars[..end].iter().collect();
            let tail: String = chars[end..].iter().collect();
            let alt_head = alts.slice(chars.len(), 0, end);
            let mut d = self.evaluate_token(
                &head,
                &alt_head,
                current_lang,
                app_bundle_id,
                opts(true, false),
            );
            if d.action != ActionType::None {
                if let Some(rep) = d.replacement.take() {
                    d.replacement = Some(format!("{rep}{tail}"));
                }
                return d;
            }
        }

        // 2. Letters-only core; edge punctuation is kept exactly as typed.
        let prefix: String = chars[..start].iter().collect();
        let core: String = chars[start..end].iter().collect();
        let suffix: String = chars[end..].iter().collect();
        let alt_core = alts.slice(chars.len(), start, end);

        let mut decision = self.evaluate_token(
            &core,
            &alt_core,
            current_lang,
            app_bundle_id,
            opts(false, true),
        );
        if let Some(rep) = decision.replacement.take() {
            decision.replacement = Some(format!("{prefix}{rep}{suffix}"));
        }
        decision
    }

    fn evaluate_token(
        &mut self,
        trimmed: &str,
        alts: &AltReadings,
        current_lang: Lang,
        app_bundle_id: &str,
        opts: EvalOpts,
    ) -> Decision {
        let char_len = trimmed.chars().count();
        if char_len == 0 {
            return Decision::none("Empty word");
        }
        if !opts.skip_guard && should_bypass_token(trimmed, current_lang) {
            return Decision::none("Guardrail: code/number/URL/path");
        }

        let active_lower = trimmed.to_lowercase();

        // Explicit user self-educated rules (`NeverConvert` / `AlwaysConvert`)
        if let Some(rule) = self.store.rules.get(&active_lower) {
            match rule.kind {
                RuleKind::NeverConvert => {
                    self.last_lang = Some(current_lang);
                    return Decision::none("Self-educated: never_convert rule");
                }
                RuleKind::AlwaysConvert => {
                    // A rule learned once or twice (often by accident: a stray ⌘ tap, or
                    // delete-and-retype) must not override a real word in the active
                    // layout, e.g. English `I` -> `Ш`.
                    let overrides_real_word = rule.hits < 3
                        && self.is_known_word(&active_lower, current_lang)
                        && !self.is_known_word(&rule.target, rule.lang);
                    if !overrides_real_word && self.is_transition_enabled(current_lang, rule.lang) {
                        let replacement = apply_casing_pattern(trimmed, &rule.target);
                        let target_lang = rule.lang;
                        self.last_lang = Some(target_lang);
                        self.store.record_auto_conversion(
                            trimmed,
                            &replacement,
                            target_lang,
                            app_bundle_id,
                        );
                        return Decision {
                            action: ActionType::ConvertLayout,
                            target_lang,
                            replacement: Some(replacement),
                            confidence: 0.99,
                            reason: "Self-educated: always_convert rule".to_string(),
                        };
                    }
                }
                RuleKind::PersonalVocab => {}
            }
        }

        // Check accidental Caps Lock in current layout (`hELLO` -> `Hello`, `пРИВЕТ` -> `Привет`)
        if let Some(fixed_caps) = detect_inverted_caps(trimmed).filter(|_| self.entitlements.allows(Feature::CapsLockFix)) {
            let fixed_lower = fixed_caps.to_lowercase();
            if self.is_known_word(&fixed_lower, current_lang)
                || self.score_phonotactics(&fixed_lower, current_lang) > 1.5
            {
                self.store.record_caps_fix();
                self.last_lang = Some(current_lang);
                return Decision {
                    action: ActionType::FixCapsLock,
                    target_lang: current_lang,
                    replacement: Some(fixed_caps),
                    confidence: 0.95,
                    reason: "Accidental Caps Lock fixed".to_string(),
                };
            }
        }

        if char_len == 1 {
            return self.evaluate_single_char(trimmed, alts, current_lang, app_bundle_id);
        }

        // If the word is already a known dictionary word or personal vocab word in the active layout, keep it!
        if self.is_known_or_personal(&active_lower, current_lang) {
            if opts.commit_none {
                self.last_lang = Some(current_lang);
                // Dictionary words don't need to be memorised as personal vocabulary;
                // only per-app language statistics are updated.
                self.store.record_app_word(current_lang, app_bundle_id);
            }
            return Decision::none("Valid dictionary word in active layout");
        }

        let active_score = self.score_word_in_lang(&active_lower, current_lang, app_bundle_id);

        // Gather enabled candidate (target_lang, candidate_word) pairs
        let candidates = self.build_candidates(trimmed, alts, current_lang);
        if candidates.is_empty() {
            return Decision::none("No enabled language pairs for current layout");
        }

        let app_caution = self
            .store
            .apps
            .get(app_bundle_id)
            .map(|a| a.caution_margin())
            .unwrap_or(0.0);

        let mut best_target: Option<(Lang, String, f64, bool)> = None;

        for (target_lang, cand_word) in candidates {
            if cand_word == trimmed || cand_word.is_empty() {
                continue;
            }
            // Judge the candidate by its letters: a key that is a letter in the active layout
            // may be punctuation in the target one (`тщцю` -> `now.`, `[jhjij/` -> `хорошо.`).
            let cand_full_lower = cand_word.to_lowercase();
            let cand_core = cand_full_lower.trim_matches(|c: char| !c.is_alphabetic());
            if cand_core.chars().count() < 2 && char_len > 1 {
                continue;
            }
            let cand_lower = cand_core.to_string();

            // Prevent spillover into English phonotactics when a word belongs to a disabled pair (FR or TR)
            if target_lang == Lang::En {
                if !self.pair_active(PairId::RuFr)
                    && is_known_fr_word(&cand_lower)
                    && !is_known_en_word(&cand_lower)
                {
                    continue;
                }
                if !self.pair_active(PairId::RuTr)
                    && is_known_tr_word(&cand_lower)
                    && !is_known_en_word(&cand_lower)
                {
                    continue;
                }
            }

            let cand_is_known = self.is_known_or_personal(&cand_lower, target_lang);
            if !cand_is_known && (opts.require_known || self.strict_known_only) {
                continue;
            }
            let cand_score = self.score_word_in_lang(&cand_lower, target_lang, app_bundle_id);

            let required_margin = if char_len == 2 {
                if !cand_is_known {
                    continue;
                }
                4.5 + app_caution
            } else if cand_is_known {
                2.2 + app_caution
            } else {
                3.8 + app_caution
            };

            let diff = cand_score - active_score;
            if diff >= required_margin && cand_score > 0.5 {
                let beats_prev = match &best_target {
                    None => true,
                    Some((_, _, best_score, best_known)) => {
                        if cand_is_known && !*best_known {
                            true
                        } else if !cand_is_known && *best_known {
                            false
                        } else {
                            cand_score > *best_score
                        }
                    }
                };
                if beats_prev {
                    best_target = Some((target_lang, cand_word, cand_score, cand_is_known));
                }
            }
        }

        if let Some((target_lang, best_word, best_score, best_known)) = best_target {
            let diff = best_score - active_score;
            let final_replacement =
                detect_inverted_caps(&best_word).unwrap_or(best_word);

            let confidence = (0.5 + (diff / 12.0)).clamp(0.55, 0.99);
            let reason = if best_known {
                format!(
                    "Dictionary match '{}' ({:?}) beats '{}' (Δ={:.1})",
                    final_replacement, target_lang, trimmed, diff
                )
            } else {
                format!(
                    "Phonotactic/morphology match '{}' ({:?}) beats '{}' (Δ={:.1})",
                    final_replacement, target_lang, trimmed, diff
                )
            };

            let final_lower = final_replacement.to_lowercase();
            self.last_lang = Some(target_lang);
            self.store.record_auto_conversion(
                trimmed,
                &final_replacement,
                target_lang,
                app_bundle_id,
            );
            self.store
                .record_accepted(&final_lower, target_lang, app_bundle_id);

            Decision {
                action: ActionType::ConvertLayout,
                target_lang,
                replacement: Some(final_replacement),
                confidence,
                reason,
            }
        } else {
            if self.typo_correction && !opts.require_known {
                if let Some(fix) =
                    self.try_fix_typo(trimmed, alts, current_lang, app_bundle_id, active_score)
                {
                    return fix;
                }
            }
            if opts.commit_none {
                self.last_lang = Some(current_lang);
                if active_score > 1.0 {
                    self.store
                        .record_accepted(&active_lower, current_lang, app_bundle_id);
                }
            }
            Decision::none("Below conversion margin")
        }
    }

    /// One-letter words (`e` -> `у`, `Ш` -> `I`) carry no evidence of their own, so they are
    /// converted only when the previous word was in the target language and the letter is
    /// a real word there while meaning nothing in the active layout.
    fn evaluate_single_char(
        &mut self,
        trimmed: &str,
        alts: &AltReadings,
        current_lang: Lang,
        app_bundle_id: &str,
    ) -> Decision {
        let active_lower = trimmed.to_lowercase();
        let context = self.last_lang;
        let none = || Decision::none("Single character (requires context or explicit rule)");

        let Some(prev) = context.filter(|&l| l != current_lang) else {
            return none();
        };
        if !trimmed.chars().all(|c| c.is_alphabetic()) || self.is_known_word(&active_lower, current_lang) {
            return none();
        }
        let Some((target_lang, cand)) = self
            .build_candidates(trimmed, alts, current_lang)
            .into_iter()
            .find(|(l, w)| *l == prev && w.chars().count() == 1 && w != trimmed)
        else {
            return none();
        };
        if !self.is_known_word(&cand.to_lowercase(), target_lang) {
            return none();
        }

        self.store
            .record_auto_conversion(trimmed, &cand, target_lang, app_bundle_id);
        Decision {
            action: ActionType::ConvertLayout,
            target_lang,
            replacement: Some(cand.clone()),
            confidence: 0.8,
            reason: format!("Single-letter word '{}' ({:?}) after {:?} context", cand, target_lang, prev),
        }
    }

    /// Spelling suggestions for `word_lower` in `lang`, best first: the host suggester
    /// (system spell checker) when installed, then the built-in lexicon.
    /// Spelling suggestions plus the host's own autocorrect choice, if it made one
    /// (marked with a leading `!` by the suggester; always one of the returned words).
    fn suggestions(&self, word_lower: &str, lang: Lang, max_edits: usize) -> (Option<String>, Vec<String>) {
        let mut out: Vec<String> = Vec::new();
        let mut preferred: Option<String> = None;
        if let Some(suggester) = self.suggester {
            if let Ok(c) = CString::new(word_lower) {
                let mut buf = vec![0u8; 2048];
                let n = suggester(c.as_ptr(), lang as i32, buf.as_mut_ptr() as *mut c_char, buf.len());
                if n > 0 && n < buf.len() {
                    if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                        for line in text.split('\n') {
                            let (marked, word) = match line.trim().strip_prefix('!') {
                                Some(w) => (true, w),
                                None => (false, line.trim()),
                            };
                            let word = word.to_lowercase();
                            if word.is_empty() {
                                continue;
                            }
                            if marked && preferred.is_none() {
                                preferred = Some(word.clone());
                            }
                            if !out.contains(&word) {
                                out.push(word);
                            }
                        }
                    }
                }
            }
        }
        for w in builtin_suggestions(word_lower, lang, max_edits) {
            if !out.contains(&w) {
                out.push(w);
            }
        }
        out.retain(|s| {
            // Letters only: never "fix" `websocket` into `web-socket`.
            s.chars().all(|c| c.is_alphabetic())
                && edit_distance(word_lower, s) <= max_edits
                && s != word_lower
        });
        let preferred = preferred.filter(|p| out.contains(p));
        (preferred, out)
    }

    /// Last resort after no plain layout conversion applied:
    /// 1. a wrong-layout word that also contains a typo (`xedcncdetn` -> `чувствует`);
    /// 2. a one-letter typo in a word typed in the right layout (`recieve` -> `receive`).
    /// Deliberately conservative: never in terminals / code editors, never for
    /// Capitalized or CAPS words in the active layout, and only unambiguous fixes.
    fn try_fix_typo(
        &mut self,
        trimmed: &str,
        alts: &AltReadings,
        current_lang: Lang,
        app_bundle_id: &str,
        active_score: f64,
    ) -> Option<Decision> {
        if is_code_app(app_bundle_id) || !self.entitlements.allows(Feature::TypoCorrection) {
            return None;
        }
        let active_lower = trimmed.to_lowercase();
        let active_len = active_lower.chars().count();
        let active_is_word = trimmed.chars().all(|c| c.is_alphabetic());

        // Corrections close to what was typed in the active layout. If any exists, the
        // user was most likely typing in the right layout with a slip of the finger.
        let same_layout = if active_is_word && active_len >= 4 {
            self.suggestions(&active_lower, current_lang, 1)
        } else {
            (None, Vec::new())
        };
        let (autocorrect, same_layout) = same_layout;

        // 1. Wrong layout + typo.
        if same_layout.is_empty() {
            let app_caution = self
                .store
                .apps
                .get(app_bundle_id)
                .map(|a| a.caution_margin())
                .unwrap_or(0.0);
            let mut best: Option<(Lang, String, f64, usize)> = None;
            for (target_lang, cand) in self.build_candidates(trimmed, alts, current_lang) {
                let cand_lower = cand.to_lowercase();
                let len = cand_lower.chars().count();
                let max_edits = max_cross_layout_edits(len);
                if max_edits == 0 || !cand_lower.chars().all(|c| c.is_alphabetic()) {
                    continue;
                }
                let (preferred, list) = self.suggestions(&cand_lower, target_lang, max_edits);
                let Some(fixed) = preferred.or_else(|| list.into_iter().next()) else {
                    continue;
                };
                let dist = edit_distance(&cand_lower, &fixed);
                let score = self.score_word_in_lang(&fixed, target_lang, app_bundle_id)
                    - dist as f64 * 1.5;
                if score - active_score < 2.2 + app_caution {
                    continue;
                }
                if best.as_ref().map(|b| score > b.2).unwrap_or(true) {
                    best = Some((target_lang, fixed, score, dist));
                }
            }
            if let Some((target_lang, fixed, score, dist)) = best {
                let replacement = apply_casing_pattern(trimmed, &fixed);
                self.last_lang = Some(target_lang);
                self.store
                    .record_auto_conversion(trimmed, &replacement, target_lang, app_bundle_id);
                return Some(Decision {
                    action: ActionType::FixTypo,
                    target_lang,
                    replacement: Some(replacement.clone()),
                    confidence: (0.5 + (score - active_score) / 14.0).clamp(0.55, 0.95),
                    reason: format!(
                        "Wrong layout + typo: '{}' ({:?}, {} edit{})",
                        replacement,
                        target_lang,
                        dist,
                        if dist == 1 { "" } else { "s" }
                    ),
                });
            }
            return None;
        }

        // 2. Typo in the active layout: lowercase words only (Capitalized ones are often
        //    names) and one clear fix — the system's autocorrect choice, else the only
        //    swapped-letter fix, else the only fix at all. How long the word must be depends
        //    on the slip: dropping a letter the user typed (`pnpm` -> `ppm`, `nastya` ->
        //    `nasty`) is the riskiest, swapped or wrong letters the safest.
        if trimmed.chars().any(|c| c.is_uppercase()) {
            return None;
        }
        let swaps: Vec<&String> = same_layout
            .iter()
            .filter(|s| is_adjacent_swap(&active_lower, s))
            .collect();
        let (fixed, backed) = if let Some(a) = autocorrect {
            (a, true)
        } else if swaps.len() == 1 {
            (swaps[0].clone(), false)
        } else if same_layout.len() == 1 {
            (same_layout[0].clone(), false)
        } else {
            return None;
        };
        let min_len = match edit_kind(&active_lower, &fixed) {
            EditKind::Swap => MIN_SAME_LAYOUT_LEN,
            // Without the system's backing, a single guess is weak evidence.
            _ if !backed => MIN_UNBACKED_EDIT_LEN,
            EditKind::Wrong => MIN_SAME_LAYOUT_LEN,
            EditKind::Missing => MIN_SAME_LAYOUT_LEN + 1,
            EditKind::Extra => MIN_UNBACKED_EDIT_LEN,
        };
        if active_len < min_len {
            return None;
        }
        self.last_lang = Some(current_lang);
        self.store
            .record_auto_conversion(trimmed, &fixed, current_lang, app_bundle_id);
        Some(Decision {
            action: ActionType::FixTypo,
            target_lang: current_lang,
            replacement: Some(fixed.clone()),
            confidence: 0.8,
            reason: format!("Typo fixed: '{}' ({:?}, 1 edit)", fixed, current_lang),
        })
    }

    pub fn is_transition_enabled(&self, from: Lang, to: Lang) -> bool {
        match (from, to) {
            (Lang::Ru, Lang::En) | (Lang::En, Lang::Ru) => self.pair_active(PairId::RuEn),
            (Lang::Ru, Lang::Fr) | (Lang::Fr, Lang::Ru) => self.pair_active(PairId::RuFr),
            (Lang::Ru, Lang::Tr) | (Lang::Tr, Lang::Ru) => self.pair_active(PairId::RuTr),
            _ => false,
        }
    }

    fn build_candidates(
        &self,
        trimmed: &str,
        alts: &AltReadings,
        current_lang: Lang,
    ) -> Vec<(Lang, String)> {
        let mut out = Vec::with_capacity(5);
        // An exact reading replaces the static keymap guesses for that language.
        let push = |out: &mut Vec<(Lang, String)>, lang: Lang, statics: &[fn(&str) -> String]| {
            if let Some(s) = alts.usable(lang, trimmed) {
                out.push((lang, s.to_string()));
                if alts.exact {
                    return;
                }
            }
            if alts.exact && alts.get(lang).is_some() {
                return; // the exact reading equals the typed text: nothing to convert into
            }
            for f in statics {
                out.push((lang, f(trimmed)));
            }
        };

        match current_lang {
            Lang::Ru => {
                if self.pair_active(PairId::RuEn) {
                    match alts.usable(Lang::En, trimmed) {
                        Some(s) => out.push((Lang::En, s.to_string())),
                        None => out.push((Lang::En, jcuken_to_qwerty(trimmed))),
                    }
                }
                if self.pair_active(PairId::RuFr) {
                    push(&mut out, Lang::Fr, &[jcuken_to_azerty, jcuken_to_qwerty]);
                }
                if self.pair_active(PairId::RuTr) {
                    push(&mut out, Lang::Tr, &[jcuken_to_turkish_q, jcuken_to_qwerty]);
                }
            }
            Lang::En => {
                if self.pair_active(PairId::RuEn) {
                    match alts.usable(Lang::Ru, trimmed) {
                        Some(s) => out.push((Lang::Ru, s.to_string())),
                        None => out.push((Lang::Ru, qwerty_to_jcuken(trimmed))),
                    }
                }
            }
            Lang::Fr => {
                if self.pair_active(PairId::RuFr) {
                    push(&mut out, Lang::Ru, &[azerty_to_jcuken, qwerty_to_jcuken]);
                }
            }
            Lang::Tr => {
                if self.pair_active(PairId::RuTr) {
                    push(&mut out, Lang::Ru, &[turkish_q_to_jcuken, qwerty_to_jcuken]);
                }
            }
        }

        out
    }

    /// An exact alternate reading turns a token that looks like code in the active layout
    /// into a plain word (`2е2` -> `été`, `^jhjij` -> `хорошо`).
    fn alt_clears_guard(&self, trimmed: &str, alts: &AltReadings, current_lang: Lang) -> bool {
        if !alts.exact || !trimmed.chars().any(|c| c.is_alphabetic()) {
            return false;
        }
        [Lang::En, Lang::Ru, Lang::Fr, Lang::Tr].into_iter().any(|lang| {
            lang != current_lang
                && self.is_transition_enabled(current_lang, lang)
                && alts.usable(lang, trimmed).is_some_and(|alt| {
                    alt.chars().filter(|c| c.is_alphabetic()).count() >= 2
                        && !should_bypass_token(alt, lang)
                })
        })
    }

    fn is_known_word(&self, word_lower: &str, lang: Lang) -> bool {
        let builtin = match lang {
            Lang::En => is_known_en_word(word_lower),
            Lang::Ru => is_known_ru_word(word_lower),
            Lang::Fr => is_known_fr_word(word_lower) || self.is_known_fr_elision(word_lower),
            Lang::Tr => is_known_tr_word(word_lower),
        };
        builtin || self.validator_accepts(word_lower, lang)
    }

    /// French elisions: `c'est`, `l'été`, `qu'il`, `aujourd'hui`.
    fn is_known_fr_elision(&self, word_lower: &str) -> bool {
        let Some((head, rest)) = word_lower.split_once(['\'', '’']) else {
            return false;
        };
        if head == "aujourd" && rest == "hui" {
            return true;
        }
        const HEADS: [&str; 14] = [
            "l", "d", "j", "m", "n", "s", "t", "c", "qu", "jusqu", "lorsqu", "puisqu", "quoiqu",
            "presqu",
        ];
        HEADS.contains(&head)
            && rest.chars().count() >= 2
            && rest.chars().all(|c| c.is_alphabetic())
            && (is_known_fr_word(rest) || self.validator_accepts(rest, Lang::Fr))
    }

    /// Asks the host dictionary. Words shorter than 3 letters are never sent: system
    /// spell checkers accept many 2-letter abbreviations ("vs", "bp", "шт") that are
    /// really mistyped words in the other layout.
    fn validator_accepts(&self, word_lower: &str, lang: Lang) -> bool {
        let Some(validator) = self.validator else {
            return false;
        };
        let mut letters = 0usize;
        for c in word_lower.chars() {
            if c.is_alphabetic() {
                letters += 1;
            } else if !matches!(c, '\'' | '’' | '-') {
                return false;
            }
        }
        if letters < 3 {
            return false;
        }
        match CString::new(word_lower) {
            Ok(c) => validator(c.as_ptr(), lang as i32),
            Err(_) => false,
        }
    }

    fn is_known_or_personal(&self, word_lower: &str, lang: Lang) -> bool {
        self.is_known_word(word_lower, lang)
            || self
                .store
                .rules
                .get(word_lower)
                .map(|r| r.kind == RuleKind::PersonalVocab && r.lang == lang)
                .unwrap_or(false)
    }

    fn score_phonotactics(&self, word_lower: &str, lang: Lang) -> f64 {
        match lang {
            Lang::En => score_en_phonotactics(word_lower),
            Lang::Ru => score_ru_phonotactics(word_lower),
            Lang::Fr => score_fr_phonotactics(word_lower),
            Lang::Tr => score_tr_phonotactics(word_lower),
        }
    }

    fn score_word_in_lang(&self, word_lower: &str, lang: Lang, app_bundle_id: &str) -> f64 {
        let mut score = 0.0;

        if self.is_known_word(word_lower, lang) {
            score += 7.5;
        }

        if let Some(rule) = self.store.rules.get(word_lower) {
            if rule.kind == RuleKind::PersonalVocab && rule.lang == lang {
                score += 8.0;
            }
        }

        score += self.score_phonotactics(word_lower, lang);

        if let Some(prev) = self.last_lang {
            if prev == lang {
                score += 0.65;
            }
        }

        if let Some(app) = self.store.apps.get(app_bundle_id) {
            let ru_bias = app.ru_prior_bias();
            match lang {
                Lang::Ru => score += ru_bias,
                Lang::En | Lang::Fr | Lang::Tr => score -= ru_bias,
            }
        }

        score
    }
}

fn should_bypass_token(token: &str, current_lang: Lang) -> bool {
    // On QWERTY, `/` and `&` are the Russian `.` and `?` keys: one of them at the very end
    // of a word is sentence punctuation (`[jhjij/` = `хорошо.`), not a path or operator.
    let body = match current_lang {
        Lang::En => token
            .strip_suffix(['/', '&'])
            .filter(|b| b.chars().count() >= 2)
            .unwrap_or(token),
        _ => token,
    };
    if body.chars().any(|c| {
        c.is_ascii_digit()
            || matches!(
                c,
                '_' | '/' | '\\' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '+' | '=' | '|'
            )
    }) {
        return true;
    }

    if token.starts_with('-') || token.ends_with('-') {
        return true;
    }

    let lower = token.to_lowercase();
    if lower.starts_with("http:")
        || lower.starts_with("https:")
        || lower.starts_with("www.")
        || lower.starts_with("ftp.")
    {
        return true;
    }

    if current_lang != Lang::Ru && token.contains('.') {
        if let Some((left, right)) = token.rsplit_once('.') {
            let right_lower = right.to_lowercase();
            let known_ext = matches!(
                right_lower.as_str(),
                "com" | "org" | "net" | "io" | "dev" | "app" | "ru" | "fr" | "tr" | "ai" | "co"
                    | "me" | "uk" | "de" | "md" | "rs" | "swift" | "py" | "js" | "ts" | "tsx"
                    | "jsx" | "json" | "toml" | "yaml" | "yml" | "html" | "css" | "sh" | "zsh"
                    | "c" | "h" | "cpp" | "hpp" | "go" | "java" | "kt" | "sql" | "db" | "sqlite"
                    | "txt" | "log" | "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "pdf"
                    | "dmg" | "zip" | "tar" | "gz" | "plist" | "xcodeproj"
            );
            if known_ext && !left.is_empty() {
                return true;
            }
        }
    }

    let chars: Vec<char> = token.chars().collect();
    if chars.len() >= 3 {
        let all_upper = chars
            .iter()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase());
        let is_inverted_caps = detect_inverted_caps(token).is_some();
        if !all_upper && !is_inverted_caps {
            let has_inner_upper = chars[1..].iter().any(|c| c.is_uppercase());
            if has_inner_upper {
                return true;
            }
        }
    }

    false
}

fn apply_casing_pattern(source: &str, target: &str) -> String {
    let src_chars: Vec<char> = source.chars().collect();
    if src_chars.iter().all(|c| !c.is_alphabetic() || c.is_uppercase())
        && src_chars.iter().any(|c| c.is_alphabetic())
    {
        return target.to_uppercase();
    }
    if let Some(first) = src_chars.first() {
        if first.is_uppercase() {
            let mut out = String::with_capacity(target.len());
            let mut tgt_iter = target.chars();
            if let Some(t0) = tgt_iter.next() {
                for u in t0.to_uppercase() {
                    out.push(u);
                }
            }
            for c in tgt_iter {
                out.push(c);
            }
            return out;
        }
    }
    target.to_string()
}
