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

mod eval;
pub use eval::*;

mod search_sites;
pub use search_sites::*;

#[cfg(feature = "cliphist")]
mod cliphist;
#[cfg(feature = "cliphist")]
pub use cliphist::*;

mod systemctl;
pub use systemctl::*;

mod history;
pub use history::*;

#[cfg(feature="linkding")]
mod linkding;
#[cfg(feature="linkding")]
pub use linkding::*;

#[cfg(feature="duckduckgo")]
mod duckduckgo;
#[cfg(feature="duckduckgo")]
pub use duckduckgo::*;


pub trait Source {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: &toml::Table, helpers: &Helpers);
    fn deinit(&mut self);
    fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher + Send + Sync>)
        -> Vec<SearchItem>;
}
