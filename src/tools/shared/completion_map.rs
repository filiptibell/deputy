use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::filter_starts_with;

/**
    An append-only map for completion purposes.

    This map handles a few things for us, notably:

    - Performance: Items are stored and referenced by prefixes,
      improving performance by 10x or more for single-character
      prefixes, and 100x or more for doubled prefixes or longer.

    - Case sensitivity: Items are matched in a case-insensitive
      manner, which is generally preferred behavior for completions.

    - Whitespace sensitivity: Item names and prefixes are trimmed
      of whitespace on both ends (leading, trailing), which is
      also generally preferred behavior for completions.

    # CAUTION

    It is extremely important that the values are
    cheap to clone, otherwise this map will create
    many duplicate values and take up a lot of memory.
*/
#[derive(Debug, Clone)]
pub struct CompletionMap<T> {
    unprefixed: Arc<Vec<T>>,
    single_char: Arc<HashMap<char, Vec<T>>>,
    double_char: Arc<HashMap<char, HashMap<char, Vec<T>>>>,
}

impl<T: Clone + AsRef<str> + 'static> CompletionMap<T> {
    const EMPTY: &[T] = &[];

    fn get(&self, key: &str) -> &[T] {
        if key.is_empty() {
            return &self.unprefixed;
        }

        let mut chars = key.chars();

        let first_char = chars.next().unwrap();
        if let Some(second_char) = chars.next() {
            self.double_char
                .get(&first_char)
                .and_then(|m| m.get(&second_char))
                .map(|v| v.as_ref())
                .unwrap_or(Self::EMPTY)
        } else {
            self.single_char
                .get(&first_char)
                .map(|v| v.as_ref())
                .unwrap_or(Self::EMPTY)
        }
    }

    /**
        Iterates over items that are guaranteed to match the given prefix.

        See the top-level documentation for additional
        details on filtering and what prefixing means.
    */
    pub fn iter(&self, prefix: impl AsRef<str>) -> impl Iterator<Item = &T> {
        let prefix: String = prefix.as_ref().trim().to_ascii_lowercase();
        self.get(prefix.as_str())
            .iter()
            .filter(move |item| filter_starts_with(item.as_ref(), prefix.as_str()))
    }
}

impl<T: Clone + AsRef<str> + 'static> FromIterator<T> for CompletionMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut unprefixed = Vec::new();
        let mut single_char = HashMap::<char, Vec<T>>::new();
        let mut double_char = HashMap::<char, HashMap<char, Vec<T>>>::new();

        let mut inserted = HashSet::new();
        for value in iter {
            let key: &str = value.as_ref().trim();
            if !inserted.insert(key.to_string()) {
                continue;
            }

            unprefixed.push(value.clone());

            let mut chars = key.chars();

            let Some(first_char) = chars.next() else {
                continue;
            };

            let first_char = first_char.to_ascii_lowercase();

            if let Some(second_char) = chars.next() {
                let second_char = second_char.to_ascii_lowercase();
                double_char
                    .entry(first_char)
                    .or_default()
                    .entry(second_char)
                    .or_default()
                    .push(value.clone());
            }

            single_char.entry(first_char).or_default().push(value);
        }

        Self {
            unprefixed: unprefixed.into(),
            single_char: single_char.into(),
            double_char: double_char.into(),
        }
    }
}
