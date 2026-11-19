//! Platform-neutral helpers for the self-updater.
//!
//! Releases are self-hosted on the LangSwitch website:
//!
//! * `<BASE>/latest.txt`             — the newest tag, e.g. `v0.3.0`
//! * `<BASE>/<tag>/<asset>`          — release assets (stable names, see below)
//! * `<BASE>/<tag>/<asset>.sha256`   — `sha256sum`-style checksum of each asset
//! * `<BASE>/latest/<asset>`         — convenience alias used by the website and installer
//!
//! The base URL can be overridden with the `LANGSWITCH_UPDATE_BASE` environment variable
//! (useful for testing a staging server).

pub const DEFAULT_BASE_URL: &str = "https://langswitch.repentandliveholy.org/download";
pub const WEBSITE_URL: &str = "https://langswitch.repentandliveholy.org";

pub const MACOS_ASSET: &str = "LangSwitch-macos-arm64.zip";
pub const WINDOWS_ASSET: &str = "langswitch-windows-x86_64.zip";
pub const LINUX_ASSET: &str = "langswitch-linux-x86_64.tar.gz";

/// Hours between automatic background checks.
pub const AUTO_CHECK_INTERVAL_HOURS: u64 = 6;

pub fn base_url() -> String {
    std::env::var("LANGSWITCH_UPDATE_BASE")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| s.starts_with("https://") || s.starts_with("http://"))
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

/// URL of the text file that names the newest release tag.
pub fn latest_url() -> String {
    format!("{}/latest.txt", base_url())
}

/// Page with downloads and release notes.
pub fn release_page_url(_tag: &str) -> String {
    format!("{WEBSITE_URL}/#download")
}

pub fn asset_url(tag: &str, asset: &str) -> String {
    format!("{}/{tag}/{asset}", base_url())
}

pub fn checksum_url(tag: &str, asset: &str) -> String {
    format!("{}.sha256", asset_url(tag, asset))
}

/// Reads the tag out of `latest.txt` (first non-empty line, must be a version).
pub fn parse_latest(text: &str) -> Option<String> {
    let tag = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let safe = tag.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'));
    if safe && parse_version(tag).is_some() {
        Some(tag.to_string())
    } else {
        None
    }
}

/// Parses `v1.2.3`, `1.2`, `1.2.3-beta` into (major, minor, patch); the pre-release suffix is ignored.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let core = s.split(['-', '+', ' ']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().map_or(Some(0), |p| p.parse().ok())?;
    let patch = parts.next().map_or(Some(0), |p| p.parse().ok())?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// `true` when `candidate` is a strictly newer version than `current`.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(c), Some(cur)) => c > cur,
        _ => false,
    }
}

/// Reads the hex digest out of a `.sha256` file (`<hex>  <name>` or just `<hex>`).
pub fn parse_checksum(text: &str) -> Option<String> {
    let hex = text.split_whitespace().next()?.to_ascii_lowercase();
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare() {
        assert_eq!(parse_version("v0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("1.2.3-beta.1"), Some((1, 2, 3)));
        assert_eq!(parse_version("latest"), None);
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(!is_newer("v0.3.0", "0.3.0"));
        assert!(!is_newer("garbage", "0.3.0"));
    }

    #[test]
    fn tag_and_checksum_parse() {
        assert_eq!(parse_latest("v0.3.0\n"), Some("v0.3.0".to_string()));
        assert_eq!(parse_latest("\n  v1.2.3  \n"), Some("v1.2.3".to_string()));
        assert_eq!(parse_latest("<html>"), None);
        assert_eq!(parse_latest("v1.0/../x"), None);
        assert!(asset_url("v0.3.0", LINUX_ASSET).ends_with("/download/v0.3.0/langswitch-linux-x86_64.tar.gz"));
        let h = "a".repeat(64);
        assert_eq!(parse_checksum(&format!("{h}  langswitch-linux-x86_64.tar.gz\n")), Some(h));
        assert_eq!(parse_checksum("nothex  file"), None);
    }
}
