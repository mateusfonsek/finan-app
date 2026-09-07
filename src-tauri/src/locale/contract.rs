//! Contract test over every locale pack in `locales/`.
//!
//! A pack is data, so a mistake in one fails silently at runtime: a typo'd
//! category key loses classification, a colour token that does not exist
//! renders colourless, a missing string quietly falls back to Portuguese.
//! Each of those becomes a test failure here.
//!
//! `pt-BR` is the reference pack: every other one must match its shape.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Value;

const REFERENCE: &str = "pt-BR";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn app_css() -> String {
    std::fs::read_to_string(repo_root().join("src").join("app.css"))
        .expect("src/app.css must be readable")
}

/// Every pack folder as (code, dir), sorted so a failure names packs in a
/// stable order.
fn packs() -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(repo_root().join("locales"))
        .expect("locales/ must exist")
        .flatten()
        .filter(|e| e.path().join("manifest.json").is_file())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    out.sort();
    assert!(!out.is_empty(), "no locale pack found under locales/");
    out
}

fn json(dir: &Path, file: &str) -> Value {
    let raw = std::fs::read_to_string(dir.join(file))
        .unwrap_or_else(|e| panic!("{}/{file}: {e}", dir.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{}/{file}: {e}", dir.display()))
}

fn category_keys(dir: &Path) -> BTreeSet<String> {
    json(dir, "categories.json")["categories"]
        .as_array()
        .expect("categories.json must hold a `categories` array")
        .iter()
        .filter_map(|c| c["key"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn manifest_is_wellformed() {
    for (code, dir) in packs() {
        let m = json(&dir, "manifest.json");

        assert_eq!(
            m["code"].as_str(),
            Some(code.as_str()),
            "{code}: manifest.code must equal the folder name"
        );
        for field in ["name", "dateLocale"] {
            assert!(
                m[field].as_str().is_some_and(|s| !s.trim().is_empty()),
                "{code}: manifest.{field} must be a non-empty string"
            );
        }

        let currency = m["currency"]["code"].as_str().unwrap_or_default();
        assert!(
            currency.len() == 3 && currency.chars().all(|c| c.is_ascii_uppercase()),
            "{code}: currency.code must be a 3-letter ISO code, got {currency:?}"
        );
        assert!(
            m["currency"]["locale"].as_str().is_some_and(|s| !s.trim().is_empty()),
            "{code}: currency.locale must be set — Intl needs it to format money"
        );

        let regex = m["taxId"]["regex"].as_str().unwrap_or_default();
        if !regex.trim().is_empty() {
            Regex::new(regex)
                .unwrap_or_else(|e| panic!("{code}: taxId.regex does not compile: {e}"));
        }

        // An unknown provider name is not an error at runtime — `for_name`
        // returns None and the locale simply has no lookup. That silence is
        // exactly what makes a typo here worth failing on.
        let provider = m["taxId"]["provider"].as_str().unwrap_or_default();
        if !provider.trim().is_empty() {
            assert!(
                crate::enrich::provider::for_name(provider).is_some(),
                "{code}: taxId.provider {provider:?} has no implementation"
            );
        }
    }
}

#[test]
fn category_keys_match_the_reference() {
    let reference = category_keys(&repo_root().join("locales").join(REFERENCE));
    assert!(!reference.is_empty(), "the reference pack declares no category");

    for (code, dir) in packs() {
        if code == REFERENCE {
            continue;
        }
        let keys = category_keys(&dir);
        let missing: Vec<&String> = reference.difference(&keys).collect();
        let extra: Vec<&String> = keys.difference(&reference).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "{code}: category keys must match {REFERENCE} exactly \
             (the keys are what rules reference, so a mismatch breaks \
             classification). Missing: {missing:?}. Extra: {extra:?}"
        );
    }
}

#[test]
fn categories_are_wellformed() {
    for (code, dir) in packs() {
        let cats = json(&dir, "categories.json");
        let list = cats["categories"].as_array().expect("categories array");
        let mut seen: BTreeSet<String> = BTreeSet::new();

        for c in list {
            let key = c["key"].as_str().unwrap_or_default();
            assert!(!key.is_empty(), "{code}: a category has no key");
            assert!(seen.insert(key.to_string()), "{code}: duplicate key {key:?}");

            assert!(
                c["name"].as_str().is_some_and(|s| !s.trim().is_empty()),
                "{code}/{key}: name must be a non-empty string"
            );

            let kind = c["kind"].as_str().unwrap_or_default();
            assert!(
                matches!(kind, "expense" | "income" | "transfer"),
                "{code}/{key}: kind must be expense|income|transfer, got {kind:?} \
                 (the DB CHECK constraint rejects anything else)"
            );
        }
    }
}

/// A `color_token` is interpolated raw into `var(...)`. A token that does not
/// exist renders with no colour and no error, which is why this is a test.
#[test]
fn color_tokens_exist_in_app_css() {
    let css = app_css();
    for (code, dir) in packs() {
        for c in json(&dir, "categories.json")["categories"]
            .as_array()
            .expect("categories array")
        {
            let key = c["key"].as_str().unwrap_or_default();
            let token = c["color_token"].as_str().unwrap_or_default();
            assert!(!token.is_empty(), "{code}/{key}: color_token is required");
            assert!(
                css.contains(&format!("{token}:")),
                "{code}/{key}: color_token {token:?} is not declared in src/app.css"
            );
        }
    }
}

#[test]
fn rule_categories_resolve_to_declared_keys() {
    for (code, dir) in packs() {
        let keys = category_keys(&dir);
        let rules = json(&dir, "rules.json");

        for (field, entries) in [
            ("cnae_map", rules["cnae_map"].as_array()),
            ("seed_rules", rules["seed_rules"].as_array()),
        ] {
            for entry in entries.into_iter().flatten() {
                let category = entry["category"].as_str().unwrap_or_default();
                assert!(
                    keys.contains(category),
                    "{code}: {field} points at category {category:?}, \
                     which categories.json does not declare"
                );
            }
        }

        let mut patterns: BTreeSet<String> = BTreeSet::new();
        for entry in rules["seed_rules"].as_array().into_iter().flatten() {
            let pattern = entry["pattern"].as_str().unwrap_or_default();
            assert!(!pattern.is_empty(), "{code}: a seed rule has an empty pattern");
            assert!(
                patterns.insert(pattern.to_string()),
                "{code}: duplicate seed pattern {pattern:?} — seed_from_pack skips \
                 a pattern that already exists, so the second one silently vanishes"
            );
        }
    }
}

#[test]
fn normalization_is_wellformed() {
    for (code, dir) in packs() {
        let norm = json(&dir, "rules.json")["normalization"].clone();

        let mask = norm["cpf_mask_regex"].as_str().unwrap_or_default();
        if !mask.trim().is_empty() {
            Regex::new(mask)
                .unwrap_or_else(|e| panic!("{code}: cpf_mask_regex does not compile: {e}"));
        }

        for rule in norm["rules"].as_array().into_iter().flatten() {
            let kind = rule["type"].as_str().unwrap_or_default();
            let prefix = rule["prefix"].as_str().unwrap_or_default();
            assert!(!prefix.is_empty(), "{code}: a normalization rule has no prefix");
            assert!(
                rule["label"].as_str().is_some_and(|s| !s.is_empty()),
                "{code}/{prefix}: normalization rule needs a label"
            );

            match kind {
                "strip" | "masked" => assert!(
                    rule["key_prefix"].as_str().is_some_and(|s| !s.is_empty()),
                    "{code}/{prefix}: a {kind} rule needs key_prefix"
                ),
                "system" => assert!(
                    rule["key"].as_str().is_some_and(|s| !s.is_empty()),
                    "{code}/{prefix}: a system rule needs key"
                ),
                other => panic!(
                    "{code}/{prefix}: normalization type must be strip|masked|system, \
                     got {other:?} — an unknown type is skipped in silence"
                ),
            }
        }
    }
}

#[test]
fn reversal_phases_are_wellformed() {
    const ROLES: [&str; 4] = ["reversal", "reversed", "refund", "refunded"];

    for (code, dir) in packs() {
        let phases = json(&dir, "rules.json")["reversals"]["phases"].clone();
        for phase in phases.as_array().into_iter().flatten() {
            let strategy = phase["strategy"].as_str().unwrap_or_default();
            let prefix = phase["prefix"].as_str().unwrap_or_default();
            assert!(!prefix.is_empty(), "{code}: a reversal phase has no prefix");

            assert!(
                phase["window_days"].as_u64().is_some_and(|d| d > 0),
                "{code}/{prefix}: window_days must be a positive integer"
            );

            for field in ["role", "counterpart_role"] {
                let role = phase[field].as_str().unwrap_or_default();
                assert!(
                    ROLES.contains(&role),
                    "{code}/{prefix}: {field} must be one of {ROLES:?}, got {role:?} \
                     — the UI builds a string key from it (import.role_<role>)"
                );
            }

            match strategy {
                "exact_remainder" | "quoted_merchant" => {}
                "counterparty_signature" => assert!(
                    phase["counterpart_prefix"].as_str().is_some_and(|s| !s.is_empty()),
                    "{code}/{prefix}: counterparty_signature needs counterpart_prefix"
                ),
                other => panic!(
                    "{code}/{prefix}: strategy must be exact_remainder|\
                     counterparty_signature|quoted_merchant, got {other:?}"
                ),
            }
        }
    }
}

/// Flattens a strings tree into dot-paths. An all-string array (`months`, ...)
/// collapses to one path, because it is a single translatable unit whose
/// length is checked separately by `calendar_arrays_have_the_expected_lengths`.
/// An array holding anything else (e.g. `about.specs`, an array of `{label,
/// value}` objects) is indexed instead, because collapsing it would hide the
/// individual leaf strings from parity — every pack would compare equal
/// regardless of how many entries or fields it actually has.
fn flatten_paths(value: &Value, prefix: &str, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_paths(v, &path, out);
            }
        }
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        Value::Array(items) => {
            if items.iter().all(Value::is_string) {
                let joined: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
                out.insert(prefix.to_string(), joined.join("\u{1}"));
            } else {
                for (i, item) in items.iter().enumerate() {
                    flatten_paths(item, &format!("{prefix}.{i}"), out);
                }
            }
        }
        _ => {}
    }
}

fn strings_of(code: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    flatten_paths(
        &json(&repo_root().join("locales").join(code), "strings.json"),
        "",
        &mut out,
    );
    out
}

/// Every `{placeholder}` in a value, as a set.
fn placeholders(text: &str) -> BTreeSet<String> {
    let re = Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("static regex");
    re.captures_iter(text)
        .map(|c| c[1].to_string())
        .collect()
}

#[test]
fn strings_keys_match_the_reference() {
    let reference = strings_of(REFERENCE);
    assert!(!reference.is_empty(), "the reference pack has no strings");

    for (code, _) in packs() {
        if code == REFERENCE {
            continue;
        }
        let theirs = strings_of(&code);
        let ref_keys: BTreeSet<&String> = reference.keys().collect();
        let their_keys: BTreeSet<&String> = theirs.keys().collect();

        let missing: Vec<&&String> = ref_keys.difference(&their_keys).collect();
        let extra: Vec<&&String> = their_keys.difference(&ref_keys).collect();
        assert!(
            missing.is_empty(),
            "{code}: missing string keys — each one falls back to {REFERENCE} \
             at runtime, in silence: {missing:?}"
        );
        assert!(extra.is_empty(), "{code}: string keys nothing reads: {extra:?}");
    }
}

/// A translator dropping `{taxId}` breaks the sentence, and nothing else says so.
#[test]
fn string_placeholders_match_the_reference() {
    let reference = strings_of(REFERENCE);

    for (code, _) in packs() {
        if code == REFERENCE {
            continue;
        }
        for (key, value) in strings_of(&code) {
            let Some(ref_value) = reference.get(&key) else {
                continue;
            };
            assert_eq!(
                placeholders(&value),
                placeholders(ref_value),
                "{code}/{key}: placeholders differ from {REFERENCE}"
            );
        }
    }
}

#[test]
fn calendar_arrays_have_the_expected_lengths() {
    for (code, dir) in packs() {
        let s = json(&dir, "strings.json");
        for (key, expected) in [("months", 12), ("months_short", 12), ("weekdays_short", 7)] {
            let len = s[key].as_array().map(Vec::len).unwrap_or(0);
            assert_eq!(len, expected, "{code}: {key} must have {expected} entries");
        }
    }
}
