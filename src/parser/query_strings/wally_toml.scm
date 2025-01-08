(table
    (bare_key) @root_name
    (pair
        (bare_key) @dependency_name
        (string) @dependency_spec
    ) @dependency_pair
    (#any-of? @root_name "dependencies" "server-dependencies" "dev-dependencies")
)
