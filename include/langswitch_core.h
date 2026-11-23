#ifndef LANGSWITCH_CORE_H
#define LANGSWITCH_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct LSEngine LSEngine;

// Language identifiers
typedef enum {
    LS_LANG_EN = 0,
    LS_LANG_RU = 1,
    LS_LANG_FR = 2,
    LS_LANG_TR = 3,
} LSLang;

// Configurable language pairs that the user can toggle ON or OFF
typedef enum {
    LS_PAIR_RU_EN = 0, // Russian <-> English (RU-EN / EN-RU)
    LS_PAIR_RU_FR = 1, // Russian <-> French  (RU-FR / FR-RU)
    LS_PAIR_RU_TR = 2, // Russian <-> Turkish (RU-TR / TR-RU)
} LSPairId;

// Action decided by the engine
typedef enum {
    LS_ACTION_NONE = 0,
    LS_ACTION_CONVERT_LAYOUT = 1,
    LS_ACTION_FIX_CAPS_LOCK = 2,
    LS_ACTION_FIX_TYPO = 3,          // replacement may also be in another layout (target_lang)
} LSActionType;

typedef struct {
    int32_t action;          // LSActionType
    int32_t target_lang;     // LSLang
    char* replacement;       // UTF-8 string (caller frees via ls_string_free), or NULL if action == NONE
    double confidence;       // 0.0 .. 1.0
    char* reason;            // Short diagnostic explanation (caller frees via ls_string_free), or NULL
} LSDecision;

// Lifecycle
LSEngine* ls_engine_new(const char* db_path);
void ls_engine_free(LSEngine* engine);

// Evaluate a completed word across enabled language pairs.
LSDecision ls_engine_evaluate(
    LSEngine* engine,
    const char* active_word,
    const char* alt_word,
    int32_t current_lang,
    const char* app_bundle_id
);

// Evaluate a word still being typed (4+ letters). Only unambiguous layout conversions;
// never spelling/caps fixes or vocabulary learning from fragments. Stop calling after a fix.
// Ownership is the same as ls_engine_evaluate (free with ls_decision_free).
LSDecision ls_engine_evaluate_prefix(
    LSEngine* engine,
    const char* active_word,
    const char* alt_word,
    int32_t current_lang,
    const char* app_bundle_id
);

// Convert arbitrary text (single word or multi-word selection) to the best enabled opposite layout.
char* ls_engine_convert_text(
    LSEngine* engine,
    const char* text,
    int32_t current_lang,
    int32_t* out_target_lang
);

// Language pair toggles (RU<->EN, RU<->FR, RU<->TR)
void ls_engine_set_pair_enabled(LSEngine* engine, int32_t pair_id, bool enabled);
bool ls_engine_is_pair_enabled(LSEngine* engine, int32_t pair_id);

// Self-education signals
void ls_engine_record_undo(
    LSEngine* engine,
    const char* original_word,
    const char* converted_word,
    int32_t original_lang,
    const char* app_bundle_id
);

void ls_engine_record_manual_convert(
    LSEngine* engine,
    const char* original_word,
    const char* converted_word,
    int32_t target_lang,
    const char* app_bundle_id
);

void ls_engine_record_accepted(
    LSEngine* engine,
    const char* word,
    int32_t lang,
    const char* app_bundle_id
);

// Optional host dictionary (e.g. the macOS spell checker). Receives a lowercase UTF-8
// word and an LSLang; returns true if it is a correctly spelled word in that language.
// While a validator is installed the engine only auto-converts into known words.
typedef bool (*LSWordValidator)(const char* word, int32_t lang);
void ls_engine_set_word_validator(LSEngine* engine, LSWordValidator validator);

// Optional host spelling suggester. Receives a lowercase UTF-8 word and an LSLang; writes
// newline-separated suggestions (best first) into `out` as a NUL-terminated UTF-8 string
// and returns the number of bytes written (0 = none). A line starting with '!' marks the host's
// own autocorrect choice (used to break ties). Without a suggester, the built-in lexicon is used.
typedef size_t (*LSWordSuggester)(const char* word, int32_t lang, char* out, size_t out_len);
void ls_engine_set_word_suggester(LSEngine* engine, LSWordSuggester suggester);

// Typo correction (on by default): fixes one-letter typos in lowercase words (4+ letters) and
// wrong-layout words that also contain a typo. Never in terminals / code editors.
void ls_engine_set_typo_correction(LSEngine* engine, bool enabled);

// Feature entitlements (everything unlocked by default). A locked feature behaves as if it were
// switched off; the user's own preference is kept and restored when the feature is unlocked.
typedef enum {
    LS_FEATURE_PAIR_RU_EN = 0,
    LS_FEATURE_PAIR_RU_FR = 1,
    LS_FEATURE_PAIR_RU_TR = 2,
    LS_FEATURE_TYPO_CORRECTION = 3,
    LS_FEATURE_CAPS_LOCK_FIX = 4
} LSFeature;

void ls_engine_set_feature_allowed(LSEngine* engine, int32_t feature, bool allowed);
bool ls_engine_is_feature_allowed(const LSEngine* engine, int32_t feature);

// Context management
void ls_engine_reset_context(LSEngine* engine);

// App exclusion management
void ls_engine_set_app_excluded(LSEngine* engine, const char* app_bundle_id, bool excluded);
bool ls_engine_is_app_excluded(LSEngine* engine, const char* app_bundle_id);

// Manual rule management from Settings UI
// rule_type: 0 = never_convert, 1 = always_convert, 2 = personal_vocab
void ls_engine_add_rule(LSEngine* engine, const char* word, int32_t rule_type, int32_t lang);
void ls_engine_remove_rule(LSEngine* engine, const char* word);
void ls_engine_clear_all_learned(LSEngine* engine);

// JSON state snapshot for the Settings & Self-Education dashboard
char* ls_engine_get_dashboard_json(LSEngine* engine);

// Memory cleanup
void ls_decision_free(LSDecision decision);
void ls_string_free(char* str);

#ifdef __cplusplus
}
#endif

#endif // LANGSWITCH_CORE_H
