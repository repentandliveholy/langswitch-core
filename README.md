# langswitch-core

The detection engine behind **[LangSwitch](https://langswitch.repentandliveholy.org)** — an automatic,
self-learning keyboard layout switcher for macOS, Windows and Linux.

Type `ghbdtn` with an English layout active and the engine tells you it should be `привет` (Russian),
in a few microseconds. It handles **RU ↔ EN**, **RU ↔ FR** (AZERTY) and **RU ↔ TR** (Turkish Q).

> This repository contains only the engine. The LangSwitch desktop apps are built on top of it and are
> distributed at [langswitch.repentandliveholy.org](https://langswitch.repentandliveholy.org).
> It is synced from the main (private) LangSwitch repository.

## Features

- **Layout detection** from multiple signals: built-in lexicons, morphology, phonotactics and per-app
  language priors. ~8 µs per word.
- **Careful by default**: punctuation stays where it belongs (`ghbdtn,` → `привет,`); code, URLs, paths,
  numbers, `camelCase` and CLI flags are never touched; an optional host dictionary makes it convert only
  into real words.
- **Typo correction** and **accidental cAPS lOCK** fixes (`hELLO` → `Hello`).
- **Early layout detection** with `Engine::evaluate_prefix` / `evaluate_prefix_with_alts` (C:
  `ls_engine_evaluate_prefix`). Compares dictionary prefixes after four letters; converts only
  with one supported target and an implausible active-language reading. Honors enabled pairs,
  entitlements, excluded/code apps and learned exceptions. Stop checking until the next word
  boundary after a conversion. Fragments do not update vocabulary or app priors.
- **Self-learning** (undo → *never convert*, manual conversion → *always convert*, personal vocabulary,
  per-app habits) persisted in SQLite (macOS/Linux; in-memory on Windows).
- **Entitlements**: a host can lock individual features (`entitlements::Feature`); everything is unlocked
  by default.
- **C ABI** (`include/langswitch_core.h`) for Swift / C / C++ hosts, plus a plain Rust API.

## Rust

```rust
use langswitch_core::{detector::Engine, ActionType, Lang};

let mut engine = Engine::new(None); // or Some("/path/to/learning.sqlite3")
let d = engine.evaluate("ghbdtn", None, Lang::En, "my.app");
assert_eq!(d.action, ActionType::ConvertLayout);
assert_eq!(d.replacement.as_deref(), Some("привет"));
```

## C

```c
#include "langswitch_core.h"

LSEngine* e = ls_engine_new(NULL);
LSDecision d = ls_engine_evaluate(e, "ghbdtn", NULL, LS_LANG_EN, "my.app");
/* d.action == LS_ACTION_CONVERT_LAYOUT, d.replacement == "привет" */
ls_decision_free(d);
ls_engine_free(e);
```

See the header for the full API. Build a static library with `cargo build --release`
(`target/release/liblangswitch_core.a`); on macOS/Linux link with `-lsqlite3`.

## Tests

```bash
cargo test --release
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this
crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
