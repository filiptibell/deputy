pub const BASE_URL_INDEX: &str = "https://index.crates.io";
pub const BASE_URL_CRATES: &str = "https://crates.io/api/v1/crates";

pub const QUERY_STRING_CRATE_SINGLE: &str = "?include=downloads,versions"; // Fetch only what we need
pub const QUERY_STRING_CRATE_MULTI: &str = "?page=1&per_page=32"; // First page only, and a reasonable amount

pub const CRAWL_MAX_INTERVAL_SECONDS: f32 = 1.25; // Max policy is once per second, let's do a bit slower
