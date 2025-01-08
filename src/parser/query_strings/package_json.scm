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
