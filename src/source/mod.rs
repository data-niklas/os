use crate::model::SearchItem;
use fuzzy_matcher::FuzzyMatcher;
use async_trait::async_trait;

mod stdin;
pub use stdin::*;

mod applications;
pub use applications::*;

mod zoxide;
pub use zoxide::*;

mod hstr;
pub use hstr::*;

#[async_trait]
pub trait Source {
    fn name(&self) -> &'static str;
    async fn init(&mut self);
    async fn deinit(&mut self);
    async fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher>) -> Vec<SearchItem>;
}
