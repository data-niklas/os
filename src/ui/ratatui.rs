use crate::ui::UI;
use crate::{model::SearchItem, os::Os};
use crossterm::event::{Event, KeyEventKind};
use crossterm::{event, execute, terminal::*};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, ListItem, Paragraph};
use relm4::gtk::prelude::ListItemExt;
use std::io;
use std::io::{stdout, Stdout};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use tui_widget_list::{List, ListState, ListableWidget};

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

struct App {
    pub prompt: String,
    pub exit: bool,
    pub items: Vec<SearchItem>,
    pub input: Input,
    pub os: Os,
    pub list: ListState,
}

impl App {
    pub fn search(&mut self) {
        let query = self.input.value();
        self.items = self.os.search(query).into_iter().take(50).collect();
    }
}

pub struct RatatuiUI {
    prompt: String,
    tui: Tui,
    app: App,
}

impl RatatuiUI {
    pub fn new(os: Os, prompt: &str) -> Self {
        let tui = init().unwrap();
        let mut list = ListState::default();
        list.select(Some(0));
        RatatuiUI {
            app: App {
                prompt: prompt.to_string(),
                exit: false,
                items: vec![],
                input: Input::new("".to_string()),
                os,
                list,
            },
            prompt: prompt.to_string(),
            tui,
        }
    }

    fn render_frame(frame: &mut Frame, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(frame.size());
        let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = app.input.visual_scroll(width as usize);
        let input = Paragraph::new(app.input.value())
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(app.prompt.clone()),
            );
        frame.render_widget(input, chunks[0]);

        let list = List::new(
            app.items
                .iter()
                .map(|item| TuiSearchItem {
                    title: item.title.clone().unwrap_or("".to_string()),
                    subtitle: item.subtitle.clone().unwrap_or("".to_string()),
                    style: Style::default(),
                })
                .collect(),
        );
        frame.render_stateful_widget(list, chunks[1], &mut app.list);
    }

    fn handle_events(app: &mut App) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Esc => {
                        app.exit = true;
                    }
                    event::KeyCode::Enter => {
                        let item = app.items.get(app.list.selected().unwrap()).unwrap();
                        app.os.select(item);
                    }
                    event::KeyCode::Down => {
                        app.list.next();
                    }
                    event::KeyCode::Up => {
                        app.list.previous();
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key_event));
                        app.list.select(Some(0));
                        app.search();
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }
}

struct TuiSearchItem {
    title: String,
    subtitle: String,
    style: Style,
}

impl ListableWidget for TuiSearchItem {
    fn size(&self, scroll_direction: &tui_widget_list::ScrollAxis) -> usize {
        3
    }
    fn highlight(self) -> Self
    where
        Self: Sized,
    {
        Self {
            style: Style::default().fg(Color::Yellow),
            ..self
        }
    }
}

impl Widget for TuiSearchItem {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::styled(self.title.clone(), self.style.clone().bold());
        let subtitle = Line::styled(self.subtitle.clone(), self.style.clone());
        buf.set_line(area.x, area.y, &title, area.width);
        buf.set_line(area.x, area.y + 1, &subtitle, area.width);
    }
}

impl UI for RatatuiUI {
    fn run(&mut self) {
        self.app.search();
        while !self.app.exit {
            self.tui
                .draw(|frame| Self::render_frame(frame, &mut self.app))
                .unwrap();
            Self::handle_events(&mut self.app).unwrap();
        }
        restore().unwrap();
    }
}
