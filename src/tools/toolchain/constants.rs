use std::collections::HashMap;

use once_cell::sync::Lazy;

pub static KNOWN_TOOLS: Lazy<HashMap<String, Vec<String>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    let mut add = |author: &str, tools: &[&str]| {
        map.insert(
            author.to_string(),
            tools.iter().map(|t| t.to_string()).collect(),
        );
    };

    /*
        HACK: The GitHub API is way too slow for autocomplete, we don't have any
        good heuristic for fetching only repositories that are tools, or only the
        users/orgs that make tools, so for now we just add in common tools manually

        Tool versions are still fetched dynamically, since there's no way
        we would be able to keep up with those, and generally a tool doesn't
        have enough releases for autocomplete to feel significantly slow
    */
    add("evaera", &["moonwave"]);
    add("filiptibell", &["lune"]);
    add("Iron-Stag-Games", &["lync"]);
    add(
        "JohnnyMorganz",
        &["luau-lsp", "StyLua", "wally-package-types"],
    );
    add("Kampfkarren", &["selene"]);
    add("Roblox", &["luau", "tarmac"]);
    add("rojo-rbx", &["remodel", "rojo", "tarmac"]);
    add("UpliftGames", &["rojo", "remodel", "tarmac", "wally"]);
    add("Quenty", &["rojo"]);

    map
});
