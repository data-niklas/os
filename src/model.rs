use relm4::gtk::gdk_pixbuf::Pixbuf;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct ClipboardContent(pub Vec<u8>);

pub struct ImmutablePixbuf(Pixbuf);
impl ImmutablePixbuf {
    pub fn new(pixbuf: Pixbuf) -> Self {
        ImmutablePixbuf(pixbuf)
    }

    pub fn borrow(&self) -> &Pixbuf {
        &self.0
    }
}
impl Clone for ImmutablePixbuf {
    fn clone(&self) -> Self {
        ImmutablePixbuf(self.0.clone())
    }
}
impl PartialEq for ImmutablePixbuf {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

unsafe impl Send for ImmutablePixbuf {}
unsafe impl Sync for ImmutablePixbuf {}

impl ClipboardContent {

    pub fn copy(self) {
        let copy_command = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");
        let mut stdin = copy_command.stdin.unwrap();
        stdin
            .write_all(self.0.as_slice())
            .expect("Failed to write to stdin");
    }
}

pub enum SelectAction {
    Noop,
    Exit,
    Print(String),
    Run(String),
    RunInTerminal(String),
    CopyToClipboard(ClipboardContent),
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
    pub icon: Option<ImmutablePixbuf>,
    pub image: Option<ImmutablePixbuf>,
    pub score: i64,
    pub action: Box<dyn Fn() -> SelectAction + Send + Sync>,
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
