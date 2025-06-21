pub fn did_you_mean<S1, S2, I>(current: S1, options: I) -> Option<String>
where
    S1: Into<String>,
    S2: Into<String>,
    I: IntoIterator<Item = S2>,
{
    let current: String = current.into();
    let options: Vec<String> = options.into_iter().map(Into::into).collect();

    let (best_score, best_option) = options
        .iter()
        .map(|s| {
            let score = strsim::jaro_winkler(&current, s);
            ((score * (u64::MAX as f64)) as u64, s)
        })
        .max_by_key(|(score, _)| *score)?;

    if (best_score as f64) / (u64::MAX as f64) >= 0.65 {
        Some(best_option.to_string())
    } else {
        None
    }
}

pub fn filter_starts_with<S1, S2>(s: S1, prefix: S2) -> bool
where
    S1: Into<String>,
    S2: Into<String>,
{
    let s: String = s.into().trim().to_ascii_lowercase();
    let prefix: String = prefix.into().trim().to_ascii_lowercase();
    s.starts_with(prefix.as_str())
}
