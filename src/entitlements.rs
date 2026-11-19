//! Feature entitlements.
//!
//! The open-source core ships with every feature unlocked. A host application may restrict
//! individual features (for example for a future premium tier) with
//! [`crate::detector::Engine::set_entitlements`]; how a host decides what is unlocked (license
//! keys, subscriptions, …) is up to the host and not part of the core.
//!
//! A locked feature behaves as if the user had switched it off, without touching the user's own
//! (persisted) preference — unlocking restores it.

/// Features that can be gated. Values are stable: they are part of the C ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Feature {
    PairRuEn = 0,
    PairRuFr = 1,
    PairRuTr = 2,
    TypoCorrection = 3,
    CapsLockFix = 4,
}

impl Feature {
    pub const ALL: [Feature; 5] = [
        Feature::PairRuEn,
        Feature::PairRuFr,
        Feature::PairRuTr,
        Feature::TypoCorrection,
        Feature::CapsLockFix,
    ];

    pub fn from_i32(v: i32) -> Option<Self> {
        Self::ALL.into_iter().find(|f| *f as i32 == v)
    }

    fn bit(self) -> u64 {
        1u64 << (self as i32 as u32)
    }
}

/// A set of unlocked features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entitlements {
    bits: u64,
}

impl Default for Entitlements {
    fn default() -> Self {
        Self::all()
    }
}

impl Entitlements {
    /// Everything unlocked (the default).
    pub const fn all() -> Self {
        Self { bits: u64::MAX }
    }

    /// Nothing unlocked.
    pub const fn none() -> Self {
        Self { bits: 0 }
    }

    pub fn allows(&self, feature: Feature) -> bool {
        self.bits & feature.bit() != 0
    }

    pub fn set(&mut self, feature: Feature, allowed: bool) {
        if allowed {
            self.bits |= feature.bit();
        } else {
            self.bits &= !feature.bit();
        }
    }

    pub fn with(mut self, feature: Feature, allowed: bool) -> Self {
        self.set(feature, allowed);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_unlocks_everything() {
        let e = Entitlements::default();
        assert!(Feature::ALL.iter().all(|f| e.allows(*f)));
        let locked = e.with(Feature::TypoCorrection, false);
        assert!(!locked.allows(Feature::TypoCorrection));
        assert!(locked.allows(Feature::PairRuFr));
        assert_eq!(Feature::from_i32(2), Some(Feature::PairRuTr));
        assert_eq!(Feature::from_i32(99), None);
    }
}
