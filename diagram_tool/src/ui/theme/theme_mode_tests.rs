use super::*;
use proptest::prelude::*;

fn mode_strategy() -> impl Strategy<Value = ThemeMode> {
    prop_oneof![
        Just(ThemeMode::System),
        Just(ThemeMode::Light),
        Just(ThemeMode::Dark),
        Just(ThemeMode::White),
    ]
}

fn scheme_strategy() -> impl Strategy<Value = ThemeScheme> {
    prop_oneof![
        Just(ThemeScheme::Light),
        Just(ThemeScheme::Dark),
        Just(ThemeScheme::White),
    ]
}

// --- B1: persisted_key returns "system" for System ---
#[test]
fn persisted_key_returns_system_for_system_mode() {
    assert_eq!(ThemeMode::System.persisted_key(), "system");
}

// --- B2: persisted_key returns "light" for Light ---
#[test]
fn persisted_key_returns_light_for_light_mode() {
    assert_eq!(ThemeMode::Light.persisted_key(), "light");
}

// --- B3: persisted_key returns "dark" for Dark ---
#[test]
fn persisted_key_returns_dark_for_dark_mode() {
    assert_eq!(ThemeMode::Dark.persisted_key(), "dark");
}

// --- B4: persisted_key returns "white" for White ---
#[test]
fn persisted_key_returns_white_for_white_mode() {
    assert_eq!(ThemeMode::White.persisted_key(), "white");
}

// --- B5: from_persisted_key parses "system" ---
#[test]
fn from_persisted_key_returns_system_for_system_string() {
    assert_eq!(
        ThemeMode::from_persisted_key("system"),
        Some(ThemeMode::System)
    );
}

// --- B6: from_persisted_key parses "light" ---
#[test]
fn from_persisted_key_returns_light_for_light_string() {
    assert_eq!(
        ThemeMode::from_persisted_key("light"),
        Some(ThemeMode::Light)
    );
}

// --- B7: from_persisted_key parses "dark" ---
#[test]
fn from_persisted_key_returns_dark_for_dark_string() {
    assert_eq!(ThemeMode::from_persisted_key("dark"), Some(ThemeMode::Dark));
}

// --- B8: from_persisted_key parses "white" ---
#[test]
fn from_persisted_key_returns_white_for_white_string() {
    assert_eq!(
        ThemeMode::from_persisted_key("white"),
        Some(ThemeMode::White)
    );
}

// --- B9: from_persisted_key returns None for unrecognized input ---
#[test]
fn from_persisted_key_returns_none_for_empty_string() {
    assert_eq!(ThemeMode::from_persisted_key(""), None);
}

#[test]
fn from_persisted_key_returns_none_for_uppercase_white() {
    assert_eq!(ThemeMode::from_persisted_key("WHITE"), None);
}

#[test]
fn from_persisted_key_returns_none_for_unknown_string() {
    assert_eq!(ThemeMode::from_persisted_key("foo"), None);
}

#[test]
fn from_persisted_key_returns_none_for_string_with_space() {
    assert_eq!(ThemeMode::from_persisted_key("Wh ite"), None);
}

// --- B9b: from_persisted_key returns None for partial-match prefixes ---
#[test]
fn from_persisted_key_returns_none_for_partial_white() {
    assert_eq!(ThemeMode::from_persisted_key("whit"), None);
}

#[test]
fn from_persisted_key_returns_none_for_partial_dark() {
    assert_eq!(ThemeMode::from_persisted_key("dar"), None);
}

#[test]
fn from_persisted_key_returns_none_for_partial_light() {
    assert_eq!(ThemeMode::from_persisted_key("ligh"), None);
}

#[test]
fn from_persisted_key_returns_none_for_partial_system() {
    assert_eq!(ThemeMode::from_persisted_key("syst"), None);
}

// --- B10: label returns "System" for System ---
#[test]
fn label_returns_system_for_system_mode() {
    assert_eq!(ThemeMode::System.label(), "System");
}

// --- B11: label returns "Light" for Light ---
#[test]
fn label_returns_light_for_light_mode() {
    assert_eq!(ThemeMode::Light.label(), "Light");
}

// --- B12: label returns "Dark" for Dark ---
#[test]
fn label_returns_dark_for_dark_mode() {
    assert_eq!(ThemeMode::Dark.label(), "Dark");
}

// --- B13: label returns "White" for White ---
#[test]
fn label_returns_white_for_white_mode() {
    assert_eq!(ThemeMode::White.label(), "White");
}

// --- B14: resolve delegates to system scheme for System mode ---
#[test]
fn resolve_returns_system_dark_when_mode_is_system_and_scheme_is_dark() {
    assert_eq!(
        ThemeMode::System.resolve(ThemeScheme::Dark),
        ThemeScheme::Dark
    );
}

#[test]
fn resolve_returns_system_light_when_mode_is_system_and_scheme_is_light() {
    assert_eq!(
        ThemeMode::System.resolve(ThemeScheme::Light),
        ThemeScheme::Light
    );
}

#[test]
fn resolve_returns_system_white_when_mode_is_system_and_scheme_is_white() {
    assert_eq!(
        ThemeMode::System.resolve(ThemeScheme::White),
        ThemeScheme::White
    );
}

// --- B15: resolve returns Light for Light mode (ignoring system) ---
#[test]
fn resolve_returns_light_when_mode_is_light_and_system_is_dark() {
    assert_eq!(
        ThemeMode::Light.resolve(ThemeScheme::Dark),
        ThemeScheme::Light
    );
}

#[test]
fn resolve_returns_light_when_mode_is_light_and_system_is_light() {
    assert_eq!(
        ThemeMode::Light.resolve(ThemeScheme::Light),
        ThemeScheme::Light
    );
}

#[test]
fn resolve_returns_light_when_mode_is_light_and_system_is_white() {
    assert_eq!(
        ThemeMode::Light.resolve(ThemeScheme::White),
        ThemeScheme::Light
    );
}

// --- B16: resolve returns Dark for Dark mode (ignoring system) ---
#[test]
fn resolve_returns_dark_when_mode_is_dark_and_system_is_light() {
    assert_eq!(
        ThemeMode::Dark.resolve(ThemeScheme::Light),
        ThemeScheme::Dark
    );
}

#[test]
fn resolve_returns_dark_when_mode_is_dark_and_system_is_dark() {
    assert_eq!(
        ThemeMode::Dark.resolve(ThemeScheme::Dark),
        ThemeScheme::Dark
    );
}

#[test]
fn resolve_returns_dark_when_mode_is_dark_and_system_is_white() {
    assert_eq!(
        ThemeMode::Dark.resolve(ThemeScheme::White),
        ThemeScheme::Dark
    );
}

// --- B17: resolve returns White for White mode (ignoring system) ---
#[test]
fn resolve_returns_white_when_mode_is_white_and_system_is_dark() {
    assert_eq!(
        ThemeMode::White.resolve(ThemeScheme::Dark),
        ThemeScheme::White
    );
}

#[test]
fn resolve_returns_white_when_mode_is_white_and_system_is_light() {
    assert_eq!(
        ThemeMode::White.resolve(ThemeScheme::Light),
        ThemeScheme::White
    );
}

#[test]
fn resolve_returns_white_when_mode_is_white_and_system_is_white() {
    assert_eq!(
        ThemeMode::White.resolve(ThemeScheme::White),
        ThemeScheme::White
    );
}

// --- B18: next() cycles System→Light→Dark→White→System ---
#[test]
fn next_cycles_system_to_light() {
    assert_eq!(ThemeMode::System.next(), ThemeMode::Light);
}

#[test]
fn next_cycles_light_to_dark() {
    assert_eq!(ThemeMode::Light.next(), ThemeMode::Dark);
}

#[test]
fn next_cycles_dark_to_white() {
    assert_eq!(ThemeMode::Dark.next(), ThemeMode::White);
}

#[test]
fn next_cycles_white_to_system() {
    assert_eq!(ThemeMode::White.next(), ThemeMode::System);
}

// --- B19: full cycle from System returns to System after 4 steps ---
#[test]
fn full_cycle_returns_to_start() {
    let modes = [
        ThemeMode::System,
        ThemeMode::Light,
        ThemeMode::Dark,
        ThemeMode::White,
    ];
    let cycled = modes
        .iter()
        .copied()
        .fold(ThemeMode::System, |m, _| m.next());
    assert_eq!(cycled, ThemeMode::System);
}

// --- B20: all 4 modes have non-empty labels ---
#[test]
fn all_modes_have_non_empty_labels() {
    let modes = [
        ThemeMode::System,
        ThemeMode::Light,
        ThemeMode::Dark,
        ThemeMode::White,
    ];
    let all_non_empty = modes.iter().copied().all(|m| !m.label().is_empty());
    assert!(all_non_empty);
}

// --- Proptest I1: persisted_key <-> from_persisted_key roundtrip ---
proptest! {
    #[test]
    fn proptest_roundtrip_persisted_key_for_all_variants(mode in mode_strategy()) {
        prop_assert_eq!(ThemeMode::from_persisted_key(mode.persisted_key()), Some(mode));
    }
}

// --- Proptest I2: persisted_key is always lowercase ASCII ---
proptest! {
    #[test]
    fn proptest_persisted_key_is_lowercase_ascii(mode in mode_strategy()) {
        prop_assert!(mode.persisted_key().chars().all(|c| c.is_ascii_lowercase()));
    }
}

// --- Proptest I3: label is always title-case ASCII ---
proptest! {
    #[test]
    fn proptest_label_is_title_case_ascii(mode in mode_strategy()) {
        let label = mode.label();
        let mut chars = label.chars();
        prop_assert!(chars.next().is_some_and(|c| c.is_ascii_uppercase()));
        prop_assert!(chars.all(|c| c.is_ascii_lowercase()));
    }
}

// --- Proptest I4: resolve(White, _) always returns White ---
proptest! {
    #[test]
    fn proptest_resolve_white_always_returns_white(system in scheme_strategy()) {
        prop_assert_eq!(ThemeMode::White.resolve(system), ThemeScheme::White);
    }
}

// --- Proptest I5: resolve(Light, _) and resolve(Dark, _) are absolute ---
proptest! {
    #[test]
    fn proptest_resolve_light_always_returns_light(system in scheme_strategy()) {
        prop_assert_eq!(ThemeMode::Light.resolve(system), ThemeScheme::Light);
    }
}

proptest! {
    #[test]
    fn proptest_resolve_dark_always_returns_dark(system in scheme_strategy()) {
        prop_assert_eq!(ThemeMode::Dark.resolve(system), ThemeScheme::Dark);
    }
}

// --- B21: next() produces all 4 labels in cycle order ---
#[test]
fn next_cycle_produces_all_four_labels_in_order() {
    let labels: Vec<&str> = std::iter::successors(Some(ThemeMode::System), |m| Some(m.next()))
        .take(4)
        .map(|m| m.label())
        .collect();
    assert_eq!(labels, vec!["System", "Light", "Dark", "White"]);
}

// --- B22: next() is involutive over 4 steps (m.next().next().next().next() == m) ---
#[test]
fn next_is_involutive_over_four_steps() {
    let modes = [
        ThemeMode::System,
        ThemeMode::Light,
        ThemeMode::Dark,
        ThemeMode::White,
    ];
    let all_return = modes
        .iter()
        .copied()
        .all(|m| m.next().next().next().next() == m);
    assert!(all_return);
}
