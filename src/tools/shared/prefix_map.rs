use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/**
    An append-only map of prefixes to values.

    This is useful for autocomplete, where we want to
    return a list of values that start with a given
    prefix, but we may have thousands of values,
    which would lead to unacceptably slow performance.

    # CAUTION

    It is extremely important that the values are
    cheap to clone, otherwise this map will create
    many duplicate values and take up a lot of memory.
*/
#[derive(Debug, Clone)]
pub struct PrefixOrderedMap<T> {
    unprefixed: Arc<Vec<T>>,
    single_char: Arc<HashMap<char, Vec<T>>>,
    double_char: Arc<HashMap<char, HashMap<char, Vec<T>>>>,
}

impl<T: Clone + 'static> PrefixOrderedMap<T> {
    const EMPTY: &[T] = &[];

    pub fn get(&self, key: &str) -> &[T] {
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
}

impl<T: Clone + AsRef<str>> FromIterator<T> for PrefixOrderedMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut unprefixed = Vec::new();
        let mut single_char = HashMap::<char, Vec<T>>::new();
        let mut double_char = HashMap::<char, HashMap<char, Vec<T>>>::new();

        let mut inserted = HashSet::new();
        for value in iter {
            let key: &str = value.as_ref();
            if !inserted.insert(key.to_string()) {
                continue;
            }

            let mut chars = key.chars();

            let Some(first_char) = chars.next() else {
                continue;
            };

            if let Some(second_char) = chars.next() {
                double_char
                    .entry(first_char)
                    .or_default()
                    .entry(second_char)
                    .or_default()
                    .push(value.clone());
            }

            unprefixed.push(value.clone());

            single_char.entry(first_char).or_default().push(value);
        }

        Self {
            unprefixed: unprefixed.into(),
            single_char: single_char.into(),
            double_char: double_char.into(),
        }
    }
}
