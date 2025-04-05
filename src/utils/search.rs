use std::sync::LazyLock;

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

pub static FUZZY_FINDER: LazyLock<SkimMatcherV2> = LazyLock::new(SkimMatcherV2::default);

pub fn fuzzy_pattern_match(pattern: &str, choice: &str) -> Vec<usize> {
    FUZZY_FINDER
        .fuzzy_indices(choice, pattern)
        .map(|(_, indices)| indices)
        .unwrap_or_default()
}
