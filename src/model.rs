use image::DynamicImage;

pub enum SelectAction {
    Noop,
    Exit,
    Print(String),
    Run(String),
    RunInTerminal(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ItemLayer {
    Bottom,
    Middle,
    Top,
}

pub struct SearchItem {
    pub id: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub icon: Option<DynamicImage>,
    pub image: Option<DynamicImage>,
    pub score: i64,
    pub action: Box<dyn Fn() -> SelectAction>,
    pub layer: ItemLayer,
    pub source: &'static str,
}

impl std::hash::Hash for SearchItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for SearchItem {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.subtitle == other.subtitle
            && self.icon == other.icon
            && self.image == other.image
            && self.score == other.score
            && self.layer == other.layer
    }
}

impl Eq for SearchItem {}

impl Ord for SearchItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.layer.cmp(&other.layer) {
            std::cmp::Ordering::Equal => self.score.cmp(&other.score),
            other => return other,
        }
    }
}

impl PartialOrd for SearchItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
