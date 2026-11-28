//! Typo correction helpers: edit distance and the built-in lexicon fallback.

use crate::lexicon::lexicon;
use crate::Lang;

/// Optimal-string-alignment distance: insertions, deletions, substitutions and
/// transpositions of adjacent letters each cost 1 (`teh` -> `the` is 1).
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[n][m]
}

/// Edits allowed when a wrong-layout word also contains a typo.
pub fn max_cross_layout_edits(len: usize) -> usize {
    match len {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}

/// Minimum length for fixing a swapped or wrong letter in a word typed in the right
/// layout (`logn` -> `long`, `дила` -> `дела`). A missing letter needs one more.
pub const MIN_SAME_LAYOUT_LEN: usize = 4;

/// Minimum length for an extra-letter fix (`pnpm` must not become `ppm`), and for any
/// non-swap fix the system spell checker did not itself choose.
pub const MIN_UNBACKED_EDIT_LEN: usize = 7;

/// Most one-edit spellings a non-swap fix may compete with (the fix included): with more,
/// the system's pick is a coin toss (`gole` -> gone / golf / gold / hole / pole...).
pub const MAX_NON_SWAP_ALTERNATIVES: usize = 2;

/// What a one-edit fix changes in the typed word.
#[derive(Debug, PartialEq, Eq)]
pub enum EditKind {
    /// Two adjacent letters swapped (`teh` -> `the`).
    Swap,
    /// One letter replaced (`дила` -> `дела`).
    Wrong,
    /// A letter was left out (`сейчс` -> `сейчас`).
    Missing,
    /// An extra letter was typed (`nastya` -> `nasty`).
    Extra,
}

pub fn edit_kind(typed: &str, fixed: &str) -> EditKind {
    let (a, b) = (typed.chars().count(), fixed.chars().count());
    if a < b {
        EditKind::Missing
    } else if a > b {
        EditKind::Extra
    } else if is_adjacent_swap(typed, fixed) {
        EditKind::Swap
    } else {
        EditKind::Wrong
    }
}

/// `true` if `b` is `a` with exactly one pair of adjacent letters swapped (`teh` / `the`).
pub fn is_adjacent_swap(a: &str, b: &str) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len() != b.len() {
        return false;
    }
    let diff: Vec<usize> = (0..a.len()).filter(|&i| a[i] != b[i]).collect();
    diff.len() == 2 && diff[1] == diff[0] + 1 && a[diff[0]] == b[diff[1]] && a[diff[1]] == b[diff[0]]
}

/// Close words from the built-in lexicon, nearest first.
pub fn builtin_suggestions(word: &str, lang: Lang, max_edits: usize) -> Vec<String> {
    let len = word.chars().count();
    let mut found: Vec<(usize, &str)> = lexicon(lang)
        .iter()
        .filter(|w| w.chars().count().abs_diff(len) <= max_edits)
        .filter_map(|w| {
            let d = edit_distance(word, w);
            (d > 0 && d <= max_edits).then_some((d, *w))
        })
        .collect();
    found.sort();
    found.into_iter().map(|(_, w)| w.to_string()).collect()
}

/// Apps where "fixing" an unknown word would break commands, code or identifiers.
pub fn is_code_app(bundle_id: &str) -> bool {
    let id = bundle_id.to_lowercase();
    [
        "terminal", "iterm", "warp", "ghostty", "kitty", "alacritty", "wezterm", "hyper",
        "xcode", "vscode", "visualstudio", "jetbrains", "sublime", "zed", "nova", "cursor",
        "windsurf", "neovim", "vim", "emacs", "tabby", "termius", "rstudio", "textmate",
        "bbedit", "coteditor", "android.studio",
    ]
    .iter()
    .any(|k| id.contains(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        assert_eq!(edit_distance("teh", "the"), 1);
        assert_eq!(edit_distance("чувстсвует", "чувствует"), 1);
        assert_eq!(edit_distance("recieve", "receive"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert!(is_adjacent_swap("recieve", "receive"));
        assert!(!is_adjacent_swap("recieve", "relieve"));
    }
}
