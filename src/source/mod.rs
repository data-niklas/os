use crate::helpers::Helpers;
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

#[cfg(feature = "cliphist")]
mod cliphist;
#[cfg(feature = "cliphist")]
pub use cliphist::*;

mod systemctl;
pub use systemctl::*;

#[cfg(feature="linkding")]
mod linkding;
#[cfg(feature="linkding")]
pub use linkding::*;


pub trait Source {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: &toml::Table, helpers: &Helpers);
    fn deinit(&mut self);
    fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher + Send + Sync>)
        -> Vec<SearchItem>;
}
