use crate::model::SearchItem;
use fuzzy_matcher::FuzzyMatcher;
use async_trait::async_trait;

mod stdin;
pub use stdin::*;

mod applications;
pub use applications::*;

#[async_trait]
pub trait Source {
    async fn name(&self) -> &str;
    async fn init(&mut self);
    async fn deinit(&mut self);
    async fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher>) -> Vec<SearchItem>;
}
