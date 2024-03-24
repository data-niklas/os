use crate::model::SearchItem;
use fuzzy_matcher::FuzzyMatcher;

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

pub trait Source {
    fn name(&self) -> &'static str;
    fn init(&mut self);
    fn deinit(&mut self);
    fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher + Send + Sync>) -> Vec<SearchItem>;
}
