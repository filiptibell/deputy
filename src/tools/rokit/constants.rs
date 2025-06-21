use std::sync::{Arc, OnceLock};

use crate::tools::shared::CompletionMap;

/**
    A statically stored author of Rokit-compatible tools.

    Stored in a text file as:

    ```
    author:a-tool,some-other-tool
    second_author:more_tools
    ```
*/
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RokitToolAuthor {
    pub name: Arc<str>,
    pub tools: Arc<[RokitToolName]>,
}

impl AsRef<str> for RokitToolAuthor {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

/**
    A statically stored Rokit-compatible tool.

    Stored in a text file as:

    ```
    author:a-tool,some-other-tool
    second_author:more_tools
    ```
*/
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RokitToolName {
    pub author: Arc<str>,
    pub name: Arc<str>,
}

impl AsRef<str> for RokitToolName {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

/*
    We bundle about as many tools as possible in a text file,
    and pre-compute them here for fast autocomplete - see the
    implementation for `PrefixOrderedMap` for more details on this.
*/

static TOP_TOOLS_ROKIT: &str = include_str!("../../../assets/top-rokit-tools.txt");
static TOP_TOOLS: OnceLock<CompletionMap<RokitToolAuthor>> = OnceLock::new();

pub fn top_rokit_tool_authors_prefixed(prefix: &str, limit: usize) -> Vec<&RokitToolAuthor> {
    let top = TOP_TOOLS.get_or_init(|| {
        TOP_TOOLS_ROKIT
            .lines()
            .filter_map(|s| {
                s.split_once(':').map(|(before, after)| {
                    let author: Arc<str> = before.to_string().into();
                    RokitToolAuthor {
                        name: Arc::clone(&author),
                        tools: after
                            .split(',')
                            .map(|name| RokitToolName {
                                author: Arc::clone(&author),
                                name: name.into(),
                            })
                            .collect::<Vec<_>>()
                            .into(),
                    }
                })
            })
            .collect::<CompletionMap<_>>()
    });

    top.iter(prefix).take(limit).collect()
}

pub fn top_rokit_tool_names_prefixed<'a>(
    author: &'a str,
    prefix: &'a str,
    limit: usize,
) -> Vec<&'a RokitToolName> {
    top_rokit_tool_authors_prefixed(author, 1)
        .first()
        .into_iter()
        .filter(|a| a.name.eq_ignore_ascii_case(author))
        .flat_map(|a| a.tools.iter().filter(|t| t.as_ref().starts_with(prefix)))
        .take(limit)
        .collect::<Vec<_>>()
}
