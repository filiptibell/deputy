pub const CARGO_TOML_DEPENDENCIES_QUERY: &str = r#"
[
    ; Regular dependencies table: [dependencies]
    (table
        (bare_key) @root_name
        [
            ; Complete dependency pairs
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
                        (pair
                            (bare_key) @misc_key
                            [
                                (string) @misc_value
                                (boolean) @misc_value
                            ]
                            (#any-of? @misc_key
                                "default-features"
                                "optional"
                                "git"
                                "rev"
                                "branch"
                            )
                        )?
                    ) @dependency_table
                ]
            ) @dependency_pair

            ; Incomplete dependency (just the key - for completions)
            (ERROR
                (bare_key) @incomplete_dependency_name
            ) @incomplete_dependency_pair
        ]
        (#any-of? @root_name
            "dependencies"
            "dev-dependencies"
            "dev_dependencies"
            "build-dependencies"
            "build_dependencies"
        )
    )

    ; Workspace/target dependencies: [workspace.dependencies] or [target.'cfg(...)'.dependencies]
    (table
        (dotted_key
            (bare_key) @table_prefix
            (#any-of? @table_prefix "workspace" "target")
            (string)? @target_spec
            (bare_key) @root_name
        )
        [
            ; Complete dependency pairs
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
                        (pair
                            (bare_key) @misc_key
                            [
                                (string) @misc_value
                                (boolean) @misc_value
                            ]
                            (#any-of? @misc_key
                                "default-features"
                                "optional"
                                "git"
                                "rev"
                                "branch"
                            )
                        )?
                    ) @dependency_table
                ]
            ) @dependency_pair

            ; Incomplete dependency (just the key - for completions)
            (ERROR
                (bare_key) @incomplete_dependency_name
            ) @incomplete_dependency_pair
        ]
        (#any-of? @root_name
            "dependencies"
            "dev-dependencies"
            "dev_dependencies"
            "build-dependencies"
            "build_dependencies"
        )
    )

    ; Named dependency sections: [dependencies.package-name]
    (table
        (dotted_key
            (bare_key) @root_name
            (bare_key) @dependency_name
        )
        [
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
            (pair
                (bare_key) @misc_key
                [
                    (string) @misc_value
                    (boolean) @misc_value
                ]
                (#any-of? @misc_key
                    "default-features"
                    "optional"
                    "git"
                    "rev"
                    "branch"
                )
            )?
        ]*
        (#any-of? @root_name
            "dependencies"
            "dev-dependencies"
            "dev_dependencies"
            "build-dependencies"
            "build_dependencies"
        )
    ) @dependency_full_capture

    ; Incomplete / hanging dependency - actually matches as *sibling* of table (?!?)
    (
        (table
            (bare_key) @root_name
            (#any-of? @root_name
                "dependencies"
                "dev-dependencies"
                "dev_dependencies"
                "build-dependencies"
                "build_dependencies"
            )
        )
        .
        (ERROR
            (bare_key) @incomplete_dependency_name
        ) @incomplete_dependency_pair
    )
]
"#;

pub const PACKAGE_JSON_DEPENDENCIES_QUERY: &str = r#"
(pair
    key: (string (string_content) @root_name)
    value: (object
        (pair
            key: (string (string_content) @dependency_name)
            value: (string (string_content) @value)
        ) @dependency_pair
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

pub const ROKIT_MANIFEST_DEPENDENCIES_QUERY: &str = r#"
(table
    (bare_key) @root_name
    (pair
        (bare_key) @dependency_name
        (string) @dependency_spec
    ) @dependency_pair
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
