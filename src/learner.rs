// The sqlite persistence layer is compiled out on Windows.
#![cfg_attr(target_os = "windows", allow(dead_code, unused_imports))]

use crate::{Lang, PairId};
use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

#[repr(C)]
struct sqlite3 {
    _private: [u8; 0],
}

#[repr(C)]
struct sqlite3_stmt {
    _private: [u8; 0],
}

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;

#[cfg_attr(target_os = "macos", link(name = "sqlite3"))]
#[cfg_attr(target_os = "linux", link(name = "sqlite3"))]
extern "C" {
    fn sqlite3_open(filename: *const c_char, pp_db: *mut *mut sqlite3) -> c_int;
    fn sqlite3_close(db: *mut sqlite3) -> c_int;
    fn sqlite3_exec(
        db: *mut sqlite3,
        sql: *const c_char,
        callback: Option<
            unsafe extern "C" fn(*mut c_void, c_int, *mut *mut c_char, *mut *mut c_char) -> c_int,
        >,
        arg: *mut c_void,
        errmsg: *mut *mut c_char,
    ) -> c_int;
    fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        z_sql: *const c_char,
        n_byte: c_int,
        pp_stmt: *mut *mut sqlite3_stmt,
        pz_tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_step(stmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_finalize(stmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_column_text(stmt: *mut sqlite3_stmt, i_col: c_int) -> *const c_char;
    fn sqlite3_column_int64(stmt: *mut sqlite3_stmt, i_col: c_int) -> i64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    NeverConvert = 0,
    AlwaysConvert = 1,
    PersonalVocab = 2,
}

#[derive(Debug, Clone)]
pub struct LearnedRule {
    pub word: String,
    pub target: String,
    pub kind: RuleKind,
    pub lang: Lang,
    pub hits: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AppProfile {
    pub bundle_id: String,
    pub en_words: u64,
    pub ru_words: u64,
    pub fr_words: u64,
    pub tr_words: u64,
    pub undo_count: u64,
    pub excluded: bool,
}

impl AppProfile {
    pub fn ru_prior_bias(&self) -> f64 {
        let non_ru = self.en_words + self.fr_words + self.tr_words;
        let total = non_ru + self.ru_words;
        if total < 5 {
            return 0.0;
        }
        let ru_ratio = (self.ru_words as f64 + 2.0) / (total as f64 + 4.0);
        (ru_ratio - 0.5) * 2.4
    }

    pub fn caution_margin(&self) -> f64 {
        if self.undo_count == 0 {
            0.0
        } else {
            (self.undo_count as f64 * 0.25).min(1.5)
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecentFix {
    pub from: String,
    pub to: String,
    pub target_lang: Lang,
    pub app_bundle_id: String,
    pub undone: bool,
}

pub struct LearningStore {
    #[cfg(not(target_os = "windows"))]
    db: *mut sqlite3,
    pub rules: HashMap<String, LearnedRule>,
    pub word_frequency: HashMap<(String, Lang), u64>,
    pub apps: HashMap<String, AppProfile>,
    pub recent_fixes: VecDeque<RecentFix>,
    pub total_auto_conversions: u64,
    pub total_manual_conversions: u64,
    pub total_undos: u64,
    pub total_caps_fixes: u64,
    pub pair_ru_en: bool,
    pub pair_ru_fr: bool,
    pub pair_ru_tr: bool,
}

unsafe impl Send for LearningStore {}

impl LearningStore {
    pub fn open(db_path: Option<&str>) -> Self {
        #[cfg(not(target_os = "windows"))]
        let db = {
            let path_str = db_path.unwrap_or(":memory:");
            let c_path = CString::new(path_str).unwrap_or_else(|_| CString::new(":memory:").unwrap());
            let mut ptr_db: *mut sqlite3 = ptr::null_mut();
            unsafe {
                if sqlite3_open(c_path.as_ptr(), &mut ptr_db) != SQLITE_OK {
                    ptr_db = ptr::null_mut();
                }
            }
            ptr_db
        };
        #[cfg(target_os = "windows")]
        let _ = db_path;

        let mut store = Self {
            #[cfg(not(target_os = "windows"))]
            db,
            rules: HashMap::new(),
            word_frequency: HashMap::new(),
            apps: HashMap::new(),
            recent_fixes: VecDeque::with_capacity(30),
            total_auto_conversions: 0,
            total_manual_conversions: 0,
            total_undos: 0,
            total_caps_fixes: 0,
            pair_ru_en: true,
            pair_ru_fr: true,
            pair_ru_tr: true,
        };

        store.init_schema();
        store.load_from_db();
        store
    }

    #[cfg(not(target_os = "windows"))]
    fn exec_sql(&self, sql: &str) {
        if self.db.is_null() {
            return;
        }
        if let Ok(c_sql) = CString::new(sql) {
            unsafe {
                sqlite3_exec(self.db, c_sql.as_ptr(), None, ptr::null_mut(), ptr::null_mut());
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn exec_sql(&self, _sql: &str) {}

    fn init_schema(&self) {
        self.exec_sql(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS learned_rules (
                 word TEXT PRIMARY KEY,
                 target TEXT NOT NULL,
                 kind INTEGER NOT NULL,
                 lang INTEGER NOT NULL,
                 hits INTEGER NOT NULL DEFAULT 1
             );
             CREATE TABLE IF NOT EXISTS app_profiles (
                 bundle_id TEXT PRIMARY KEY,
                 en_words INTEGER NOT NULL DEFAULT 0,
                 ru_words INTEGER NOT NULL DEFAULT 0,
                 undo_count INTEGER NOT NULL DEFAULT 0,
                 excluded INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS stats (
                 key TEXT PRIMARY KEY,
                 val INTEGER NOT NULL DEFAULT 0
             );",
        );
    }

    #[cfg(not(target_os = "windows"))]
    fn load_from_db(&mut self) {
        if self.db.is_null() {
            return;
        }

        unsafe {
            let sql = CString::new("SELECT word, target, kind, lang, hits FROM learned_rules").unwrap();
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            if sqlite3_prepare_v2(self.db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()) == SQLITE_OK {
                while sqlite3_step(stmt) == SQLITE_ROW {
                    let w_ptr = sqlite3_column_text(stmt, 0);
                    let t_ptr = sqlite3_column_text(stmt, 1);
                    if !w_ptr.is_null() && !t_ptr.is_null() {
                        let word = CStr::from_ptr(w_ptr).to_string_lossy().into_owned();
                        let target = CStr::from_ptr(t_ptr).to_string_lossy().into_owned();
                        let kind_int = sqlite3_column_int64(stmt, 2);
                        let lang_int = sqlite3_column_int64(stmt, 3) as i32;
                        let hits = sqlite3_column_int64(stmt, 4).max(1) as u64;
                        let kind = match kind_int {
                            0 => RuleKind::NeverConvert,
                            1 => RuleKind::AlwaysConvert,
                            _ => RuleKind::PersonalVocab,
                        };
                        let lang = Lang::from_i32(lang_int);
                        self.rules.insert(
                            word.clone(),
                            LearnedRule {
                                word,
                                target,
                                kind,
                                lang,
                                hits,
                            },
                        );
                    }
                }
                sqlite3_finalize(stmt);
            }

            let sql = CString::new(
                "SELECT bundle_id, en_words, ru_words, undo_count, excluded FROM app_profiles",
            )
            .unwrap();
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            if sqlite3_prepare_v2(self.db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()) == SQLITE_OK {
                while sqlite3_step(stmt) == SQLITE_ROW {
                    let b_ptr = sqlite3_column_text(stmt, 0);
                    if !b_ptr.is_null() {
                        let bundle_id = CStr::from_ptr(b_ptr).to_string_lossy().into_owned();
                        let en_words = sqlite3_column_int64(stmt, 1).max(0) as u64;
                        let ru_words = sqlite3_column_int64(stmt, 2).max(0) as u64;
                        let undo_count = sqlite3_column_int64(stmt, 3).max(0) as u64;
                        let excluded = sqlite3_column_int64(stmt, 4) != 0;
                        self.apps.insert(
                            bundle_id.clone(),
                            AppProfile {
                                bundle_id,
                                en_words,
                                ru_words,
                                fr_words: 0,
                                tr_words: 0,
                                undo_count,
                                excluded,
                            },
                        );
                    }
                }
                sqlite3_finalize(stmt);
            }

            let sql = CString::new("SELECT key, val FROM stats").unwrap();
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            if sqlite3_prepare_v2(self.db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()) == SQLITE_OK {
                while sqlite3_step(stmt) == SQLITE_ROW {
                    let k_ptr = sqlite3_column_text(stmt, 0);
                    if !k_ptr.is_null() {
                        let key = CStr::from_ptr(k_ptr).to_string_lossy();
                        let val = sqlite3_column_int64(stmt, 1);
                        match key.as_ref() {
                            "auto_conversions" => self.total_auto_conversions = val.max(0) as u64,
                            "manual_conversions" => self.total_manual_conversions = val.max(0) as u64,
                            "undos" => self.total_undos = val.max(0) as u64,
                            "caps_fixes" => self.total_caps_fixes = val.max(0) as u64,
                            "pair_ru_en" => self.pair_ru_en = val != 0,
                            "pair_ru_fr" => self.pair_ru_fr = val != 0,
                            "pair_ru_tr" => self.pair_ru_tr = val != 0,
                            _ => {}
                        }
                    }
                }
                sqlite3_finalize(stmt);
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn load_from_db(&mut self) {}

    fn persist_rule(&self, rule: &LearnedRule) {
        let escaped_word = sql_escape(&rule.word);
        let escaped_target = sql_escape(&rule.target);
        let sql = format!(
            "INSERT OR REPLACE INTO learned_rules (word, target, kind, lang, hits) VALUES ('{}', '{}', {}, {}, {});",
            escaped_word,
            escaped_target,
            rule.kind as i32,
            rule.lang as i32,
            rule.hits
        );
        self.exec_sql(&sql);
    }

    fn persist_app(&self, app: &AppProfile) {
        let escaped_bundle = sql_escape(&app.bundle_id);
        let sql = format!(
            "INSERT OR REPLACE INTO app_profiles (bundle_id, en_words, ru_words, undo_count, excluded) VALUES ('{}', {}, {}, {}, {});",
            escaped_bundle,
            app.en_words + app.fr_words + app.tr_words,
            app.ru_words,
            app.undo_count,
            if app.excluded { 1 } else { 0 }
        );
        self.exec_sql(&sql);
    }

    fn persist_stat(&self, key: &str, val: u64) {
        let sql = format!(
            "INSERT OR REPLACE INTO stats (key, val) VALUES ('{}', {});",
            sql_escape(key),
            val
        );
        self.exec_sql(&sql);
    }

    pub fn set_pair_enabled(&mut self, pair: PairId, enabled: bool) {
        let v = if enabled { 1 } else { 0 };
        match pair {
            PairId::RuEn => {
                self.pair_ru_en = enabled;
                self.persist_stat("pair_ru_en", v);
            }
            PairId::RuFr => {
                self.pair_ru_fr = enabled;
                self.persist_stat("pair_ru_fr", v);
            }
            PairId::RuTr => {
                self.pair_ru_tr = enabled;
                self.persist_stat("pair_ru_tr", v);
            }
        }
    }

    pub fn is_pair_enabled(&self, pair: PairId) -> bool {
        match pair {
            PairId::RuEn => self.pair_ru_en,
            PairId::RuFr => self.pair_ru_fr,
            PairId::RuTr => self.pair_ru_tr,
        }
    }

    pub fn record_auto_conversion(
        &mut self,
        from: &str,
        to: &str,
        target_lang: Lang,
        app_bundle_id: &str,
    ) {
        self.total_auto_conversions += 1;
        self.persist_stat("auto_conversions", self.total_auto_conversions);
        if self.recent_fixes.len() >= 25 {
            self.recent_fixes.pop_back();
        }
        self.recent_fixes.push_front(RecentFix {
            from: from.to_string(),
            to: to.to_string(),
            target_lang,
            app_bundle_id: app_bundle_id.to_string(),
            undone: false,
        });
    }

    pub fn record_caps_fix(&mut self) {
        self.total_caps_fixes += 1;
        self.persist_stat("caps_fixes", self.total_caps_fixes);
    }

    pub fn record_undo(
        &mut self,
        original_word: &str,
        converted_word: &str,
        original_lang: Lang,
        app_bundle_id: &str,
    ) {
        let key = original_word.to_lowercase();
        if key.is_empty() {
            return;
        }
        self.total_undos += 1;
        self.persist_stat("undos", self.total_undos);

        if let Some(front) = self.recent_fixes.front_mut() {
            if front.from.eq_ignore_ascii_case(original_word) || front.to == converted_word {
                front.undone = true;
            }
        }

        let hits = self.rules.get(&key).map(|r| r.hits + 1).unwrap_or(1);
        let rule = LearnedRule {
            word: key.clone(),
            target: converted_word.to_lowercase(),
            kind: RuleKind::NeverConvert,
            lang: original_lang,
            hits,
        };
        self.persist_rule(&rule);
        self.rules.insert(key, rule);

        if !app_bundle_id.is_empty() {
            let app = self
                .apps
                .entry(app_bundle_id.to_string())
                .or_insert_with(|| AppProfile {
                    bundle_id: app_bundle_id.to_string(),
                    ..Default::default()
                });
            app.undo_count += 1;
            let app_clone = app.clone();
            self.persist_app(&app_clone);
        }
    }

    pub fn record_manual_convert(
        &mut self,
        original_word: &str,
        converted_word: &str,
        target_lang: Lang,
        app_bundle_id: &str,
    ) {
        let key = original_word.to_lowercase();
        if key.is_empty() {
            return;
        }
        self.total_manual_conversions += 1;
        self.persist_stat("manual_conversions", self.total_manual_conversions);

        // A new `x -> y` rule contradicts an older `y -> x` rule (e.g. `ш -> i` vs `i -> ш`):
        // the latest intent wins, otherwise the pair ping-pongs and breaks both words.
        let inverse = converted_word.to_lowercase();
        if inverse != key
            && self
                .rules
                .get(&inverse)
                .map(|r| r.kind == RuleKind::AlwaysConvert && r.target == key)
                .unwrap_or(false)
        {
            self.remove_rule(&inverse);
        }

        let hits = self.rules.get(&key).map(|r| r.hits + 1).unwrap_or(1);
        let rule = LearnedRule {
            word: key.clone(),
            target: converted_word.to_lowercase(),
            kind: RuleKind::AlwaysConvert,
            lang: target_lang,
            hits,
        };
        self.persist_rule(&rule);
        self.rules.insert(key, rule);

        self.record_accepted(converted_word, target_lang, app_bundle_id);
    }

    pub fn record_accepted(&mut self, word: &str, lang: Lang, app_bundle_id: &str) {
        let key = word.to_lowercase();
        if key.chars().count() >= 3 {
            let count = self
                .word_frequency
                .entry((key.clone(), lang))
                .and_modify(|c| *c += 1)
                .or_insert(1);

            if *count >= 3 && !self.rules.contains_key(&key) {
                let rule = LearnedRule {
                    word: key.clone(),
                    target: key.clone(),
                    kind: RuleKind::PersonalVocab,
                    lang,
                    hits: *count,
                };
                self.persist_rule(&rule);
                self.rules.insert(key, rule);
            } else if let Some(existing) = self.rules.get_mut(&key) {
                if existing.kind == RuleKind::PersonalVocab && existing.lang == lang {
                    existing.hits += 1;
                    let clone = existing.clone();
                    self.persist_rule(&clone);
                }
            }
        }

        self.record_app_word(lang, app_bundle_id);
    }

    /// Updates only the per-app language statistics (no vocabulary learning).
    pub fn record_app_word(&mut self, lang: Lang, app_bundle_id: &str) {
        if app_bundle_id.is_empty() {
            return;
        }
        let app = self
            .apps
            .entry(app_bundle_id.to_string())
            .or_insert_with(|| AppProfile {
                bundle_id: app_bundle_id.to_string(),
                ..Default::default()
            });
        match lang {
            Lang::En => app.en_words += 1,
            Lang::Ru => app.ru_words += 1,
            Lang::Fr => app.fr_words += 1,
            Lang::Tr => app.tr_words += 1,
        }
        let app_clone = app.clone();
        self.persist_app(&app_clone);
    }

    pub fn set_app_excluded(&mut self, app_bundle_id: &str, excluded: bool) {
        if app_bundle_id.is_empty() {
            return;
        }
        let app = self
            .apps
            .entry(app_bundle_id.to_string())
            .or_insert_with(|| AppProfile {
                bundle_id: app_bundle_id.to_string(),
                ..Default::default()
            });
        app.excluded = excluded;
        let clone = app.clone();
        self.persist_app(&clone);
    }

    pub fn is_app_excluded(&self, app_bundle_id: &str) -> bool {
        self.apps
            .get(app_bundle_id)
            .map(|a| a.excluded)
            .unwrap_or(false)
    }

    pub fn add_rule(&mut self, word: &str, target: &str, kind: RuleKind, lang: Lang) {
        let key = word.trim().to_lowercase();
        if key.is_empty() {
            return;
        }
        let rule = LearnedRule {
            word: key.clone(),
            target: target.trim().to_lowercase(),
            kind,
            lang,
            hits: 1,
        };
        self.persist_rule(&rule);
        self.rules.insert(key, rule);
    }

    pub fn remove_rule(&mut self, word: &str) {
        let key = word.trim().to_lowercase();
        self.rules.remove(&key);
        let sql = format!("DELETE FROM learned_rules WHERE word = '{}';", sql_escape(&key));
        self.exec_sql(&sql);
    }

    pub fn clear_all(&mut self) {
        self.rules.clear();
        self.word_frequency.clear();
        self.apps.clear();
        self.recent_fixes.clear();
        self.total_auto_conversions = 0;
        self.total_manual_conversions = 0;
        self.total_undos = 0;
        self.total_caps_fixes = 0;
        self.exec_sql(
            "DELETE FROM learned_rules;
             DELETE FROM app_profiles;
             DELETE FROM stats;",
        );
    }

    pub fn to_dashboard_json(&self) -> String {
        let mut rules_vec: Vec<&LearnedRule> = self.rules.values().collect();
        rules_vec.sort_by(|a, b| b.hits.cmp(&a.hits).then_with(|| a.word.cmp(&b.word)));

        let mut rules_json = String::from("[");
        for (i, r) in rules_vec.iter().enumerate() {
            if i > 0 {
                rules_json.push(',');
            }
            rules_json.push_str(&format!(
                "{{\"word\":\"{}\",\"target\":\"{}\",\"kind\":{},\"lang\":{},\"hits\":{}}}",
                json_escape(&r.word),
                json_escape(&r.target),
                r.kind as i32,
                r.lang as i32,
                r.hits
            ));
        }
        rules_json.push(']');

        let mut apps_vec: Vec<&AppProfile> = self.apps.values().collect();
        apps_vec.sort_by(|a, b| {
            (b.en_words + b.ru_words + b.fr_words + b.tr_words)
                .cmp(&(a.en_words + a.ru_words + a.fr_words + a.tr_words))
                .then_with(|| a.bundle_id.cmp(&b.bundle_id))
        });

        let mut apps_json = String::from("[");
        for (i, a) in apps_vec.iter().enumerate() {
            if i > 0 {
                apps_json.push(',');
            }
            apps_json.push_str(&format!(
                "{{\"bundleId\":\"{}\",\"enWords\":{},\"ruWords\":{},\"undoCount\":{},\"excluded\":{}}}",
                json_escape(&a.bundle_id),
                a.en_words + a.fr_words + a.tr_words,
                a.ru_words,
                a.undo_count,
                if a.excluded { "true" } else { "false" }
            ));
        }
        apps_json.push(']');

        let mut fixes_json = String::from("[");
        for (i, f) in self.recent_fixes.iter().enumerate() {
            if i > 0 {
                fixes_json.push(',');
            }
            fixes_json.push_str(&format!(
                "{{\"from\":\"{}\",\"to\":\"{}\",\"targetLang\":{},\"appBundleId\":\"{}\",\"undone\":{}}}",
                json_escape(&f.from),
                json_escape(&f.to),
                f.target_lang as i32,
                json_escape(&f.app_bundle_id),
                if f.undone { "true" } else { "false" }
            ));
        }
        fixes_json.push(']');

        format!(
            "{{\"stats\":{{\"autoConversions\":{},\"manualConversions\":{},\"undos\":{},\"capsFixes\":{},\"learnedRulesCount\":{}}},\"pairs\":{{\"ruEn\":{},\"ruFr\":{},\"ruTr\":{}}},\"rules\":{},\"apps\":{},\"recentFixes\":{}}}",
            self.total_auto_conversions,
            self.total_manual_conversions,
            self.total_undos,
            self.total_caps_fixes,
            self.rules.len(),
            if self.pair_ru_en { "true" } else { "false" },
            if self.pair_ru_fr { "true" } else { "false" },
            if self.pair_ru_tr { "true" } else { "false" },
            rules_json,
            apps_json,
            fixes_json
        )
    }
}

#[cfg(not(target_os = "windows"))]
impl Drop for LearningStore {
    fn drop(&mut self) {
        if !self.db.is_null() {
            unsafe {
                sqlite3_close(self.db);
            }
            self.db = ptr::null_mut();
        }
    }
}

fn sql_escape(s: &str) -> String {
    s.replace('\'', "''")
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {}
            other => out.push(other),
        }
    }
    out
}
