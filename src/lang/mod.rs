use std::ops::Range;

pub mod json;
pub mod toml;

pub trait LangValue: Clone {
    fn span(&self) -> Range<usize>;
    fn source(&self) -> &str;
}

pub trait LangBool: LangValue {
    fn value(&self) -> bool;
}

pub trait LangFloat: LangValue {
    fn value(&self) -> f64;
}

pub trait LangString: LangValue {
    fn value(&self) -> &str;
}
