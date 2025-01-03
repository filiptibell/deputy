#![allow(dead_code)]

pub struct MarkdownBuilder {
    lines: Vec<String>,
}

impl MarkdownBuilder {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn br(&mut self) {
        self.lines.push(String::new());
    }

    pub fn p(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    pub fn h1(&mut self, line: impl Into<String>) {
        self.p(format!("# {}", line.into()));
    }

    pub fn h2(&mut self, line: impl Into<String>) {
        self.p(format!("## {}", line.into()));
    }

    pub fn h3(&mut self, line: impl Into<String>) {
        self.p(format!("### {}", line.into()));
    }

    pub fn a(&mut self, text: impl Into<String>, link: impl Into<String>) {
        self.p(format!("- [{}]({})", text.into(), link.into()));
    }

    pub fn ver(&mut self, version: impl Into<String>) {
        self.p(format!("Version **{}**", version.into()));
    }

    pub fn extend_last(&mut self, suffix: impl AsRef<str>) {
        let last = self.lines.last_mut().unwrap();
        last.push_str(suffix.as_ref());
    }

    pub fn build(mut self) -> String {
        self.lines.push(String::new()); // Ensure last line is empty
        self.lines.join("\n")
    }
}
