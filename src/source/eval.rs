use crate::helpers::Helpers;
use crate::model::{ClipboardContent, SearchItem};
use crate::source::Source;
use eval::{Expr};
use fuzzy_matcher::FuzzyMatcher;


pub struct EvalSource {}

impl EvalSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl Source for EvalSource {
    fn name(&self) -> &'static str {
        "eval"
    }

    fn init(&mut self, _config: &toml::Table, _helpers: &Helpers) {}

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        _matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        let expr = Expr::new(query);
        let result = expr.exec();
        if let Ok(value) = result {
            let value_text = value.to_string();
            let action_value_text = value_text.clone();
            let item = SearchItem {
                id: self.name().to_string(),
                title: Some(value_text),
                subtitle: Some(String::from("eval")),
                icon: None,
                image: None,
                score: 1,
                source: self.name(),
                layer: crate::model::ItemLayer::Top,
                action: Box::new(move |os| {
                    let text_bytes = action_value_text.clone().into_bytes();
                    os.copy_to_clipboard(ClipboardContent(text_bytes));
                    true
                }),
            };
            return vec![item];
        } else {
            return vec![];
        }
    }
}
