use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum ToolName {
    Aftman,
    Foreman,
    Wally,
}

impl FromStr for ToolName {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "aftman" | "aftman.toml" => Ok(Self::Aftman),
            "foreman" | "foreman.toml" => Ok(Self::Foreman),
            "wally" | "wally.toml" => Ok(Self::Wally),
            _ => Err("Unknown tool"),
        }
    }
}
