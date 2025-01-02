pub const CARGO_TOML_DEPENDENCIES_QUERY: &str = r#"
(table
    (bare_key) @root_name
    (pair
        (bare_key) @dependency_name
        [
            (string) @version
            (inline_table
                (pair
                    (bare_key) @version_key
                    (string) @version
                    (#eq? @version_key "version")
                )?
                (pair
                    (bare_key) @features_key
                    (array) @features_array
                    (#eq? @features_key "features")
                )?
            )
        ]
    ) @dependency_pair
    (#any-of? @root_name
	    "dependencies"
	    "dev-dependencies"
	    "build-dependencies"
    )
)
"#;

pub const PACKAGE_JSON_DEPENDENCIES_QUERY: &str = r#"
(pair
    key: (string (string_content) @root_name)
    value: (object
        (pair
            key: (string (string_content) @dependency_name)
            value: (string (string_content) @value)
        )
    )+
    (#any-of? @root_name
        "dependencies"
        "devDependencies"
        "peerDependencies"
        "optionalDependencies"
        "bundledDependencies"
        "bundleDependencies"
    )
)+
"#;

pub const ROKIT_MANIFEST_TOOLS_QUERY: &str = r#"
(table
    (bare_key) @root_name
    (pair
        (bare_key) @tool_name
        (string) @tool_spec
    ) @tool_pair
    (#eq? @root_name "tools")
)
"#;

pub const WALLY_MANIFEST_DEPENDENCIES_QUERY: &str = r#"
(table
    (bare_key) @root_name
    (pair
        (bare_key) @dependency_name
        (string) @dependency_spec
    ) @dependency_pair
    (#any-of? @root_name "dependencies" "server-dependencies" "dev-dependencies")
)
"#;
