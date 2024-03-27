use crate::model::SearchItem;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;

mod stdin;
pub use stdin::*;

mod applications;
pub use applications::*;

mod zoxide;
pub use zoxide::*;

mod hstr;
pub use hstr::*;

mod cliphist;
pub use cliphist::*;

mod systemctl;
pub use systemctl::*;

mod linkding;
pub use linkding::*;

pub trait Source {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: &toml::Table);
    fn deinit(&mut self);
    fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher + Send + Sync>)
        -> Vec<SearchItem>;
}
