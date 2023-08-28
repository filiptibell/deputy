use super::*;

const NO_TOOLS: &str = r#"
[tools]
"#;

#[test]
fn no_tools() {
    let parsed = Manifest::parse(NO_TOOLS).expect("Failed to parse manifest");
    assert_eq!(parsed.tools_map.tools.len(), 0)
}

const ONE_TOOL: &str = r#"
[tools]
tool-name = "author/tool@1.2.3"
"#;

#[test]
fn one_tool() {
    let parsed = Manifest::parse(ONE_TOOL).expect("Failed to parse manifest");
    assert_eq!(parsed.tools_map.tools.len(), 1)
}

const TWO_TOOLS: &str = r#"
[tools]
tool-name = "author/tool@1.2.3"
epic_pre_rc_1 = 'epic-person/epic-tool@0.0.1-prerelease.rc.1'
"#;

#[test]
fn two_tools() {
    let parsed = Manifest::parse(TWO_TOOLS).expect("Failed to parse manifest");
    assert_eq!(parsed.tools_map.tools.len(), 2)
}

const TOOLS_AND_EXTRAS: &str = r#"
# Tools managed by the tool manager
[tools]

  tool-name = "author/tool@1.2.3" # Comment about a tool

epic_pre_rc_1 = 'epic-person/epic-tool@0.0.1-prerelease.rc.1'
"#;

#[test]
fn tools_and_extras() {
    let parsed = Manifest::parse(TOOLS_AND_EXTRAS).expect("Failed to parse manifest");
    assert_eq!(parsed.tools_map.tools.len(), 2)
}
