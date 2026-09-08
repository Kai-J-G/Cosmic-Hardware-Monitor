//! Localization, backed by Fluent translation files under `i18n/`.
//!
//! Those files are embedded into the binary at build time, so a translation is
//! shipped with the applet rather than installed alongside it.
//!
//! Reach for strings through the [`fl!`](crate::fl) macro:
//!
//! ```ignore
//! text::title3(fl!("storage"))
//! ```
//!
//! The macro resolves the id against `i18n/en/…ftl` **at compile time**, so a
//! typo or a missing translation is a build error rather than a blank label.

use std::sync::LazyLock;

use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::unic_langid::LanguageIdentifier;
use i18n_embed::{DefaultLocalizer, LanguageLoader, Localizer};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    // English is compiled in as the fallback, so every id always resolves.
    loader
        .load_fallback_language(&Localizations)
        .expect("i18n/en is embedded at build time and must load");

    // Fluent wraps interpolated values in Unicode bidi isolation marks, which
    // protect right-to-left text but render as boxes in fonts that lack them.
    // The readings here are numbers inside short labels, where the marks buy
    // nothing and the risk of visible tofu is real.
    loader.set_use_isolating(false);

    loader
});

/// Selects the best available translation for the languages the desktop asks
/// for, falling back to English for anything untranslated.
pub fn init(requested_languages: &[LanguageIdentifier]) {
    let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations);

    // A failure here just means the user keeps English, so it is not fatal.
    if let Err(why) = localizer.select(requested_languages) {
        eprintln!("failed to load localizations: {why}");
    }

    // Selecting a language builds fresh Fluent bundles, which do not inherit
    // the setting applied to the fallback, so re-apply it here. Without this
    // every translated string carrying a value would show isolation marks.
    LANGUAGE_LOADER.set_use_isolating(false);
}

/// Looks up a localized string by its id in `i18n/`.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn language(tag: &str) -> LanguageIdentifier {
        tag.parse().expect("valid language tag")
    }

    /// The message ids defined in one translation file.
    fn ids(language: &str) -> BTreeSet<String> {
        let path = format!("{}/i18n/{language}/cosmic_ext_hardware_monitor.ftl", env!("CARGO_MANIFEST_DIR"));
        let contents = std::fs::read_to_string(path).expect("translation file exists");

        contents
            .lines()
            .filter_map(|line| line.split_once(" ="))
            .map(|(id, _)| id.trim().to_string())
            .filter(|id| !id.starts_with('#') && !id.is_empty())
            .collect()
    }

    fn languages() -> Vec<String> {
        let dir = format!("{}/i18n", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_dir(dir)
            .expect("i18n directory exists")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect()
    }

    /// A translation missing an id falls back to English mid-sentence, so every
    /// language has to define the full set.
    #[test]
    fn every_language_defines_the_same_ids() {
        let english = ids("en");
        assert!(!english.is_empty(), "no ids parsed from i18n/en");

        for language in languages() {
            let defined = ids(&language);
            let missing: Vec<_> = english.difference(&defined).collect();
            assert!(missing.is_empty(), "i18n/{language} is missing: {missing:?}");
        }
    }

    #[test]
    fn selecting_a_language_changes_the_strings() {
        init(&[language("fr")]);
        assert_eq!(fl!("settings"), "Paramètres");
        assert_eq!(fl!("all-cores", count = 16), "Les 16 cœurs");

        init(&[language("en")]);
        assert_eq!(fl!("settings"), "Settings");
        assert_eq!(fl!("all-cores", count = 16), "All 16 Cores");
    }
}
