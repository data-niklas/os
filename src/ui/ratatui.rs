use crate::ui::UI;
use crate::{model::SearchItem, os::Os};
use crossterm::event::{Event, KeyEventKind};
use crossterm::{event, execute, terminal::*};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState};
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
    pub items: Vec<SearchItem>,
    pub input: Input,
    pub os: Os,
    pub list: ListState,
    pub scroll_state: ScrollbarState,
}

impl App {
    pub fn search(&mut self) {
        let query = self.input.value();
        self.list.select(Some(0));
        self.items = self
            .os
            .search(query)
            .into_iter()
            .take(self.os.config.maximum_list_item_count)
            .collect();
        self.scroll_state = self
            .scroll_state
            .position(0)
            .content_length(self.items.len());
    }

    pub fn exit(&self) {
        restore().unwrap();
    }
}

pub struct RatatuiUI {
    tui: Tui,
    app: App,
}

impl RatatuiUI {
    pub fn new(os: Os, prompt: &str) -> Self {
        let tui = init().unwrap();
        let mut list = ListState::default().circular(false);
        list.select(Some(0));
        let scroll_state = ScrollbarState::default();
        RatatuiUI {
            app: App {
                prompt: prompt.to_string(),
                items: vec![],
                input: Input::new("".to_string()),
                os,
                list,
                scroll_state,
            },
            tui,
        }
    }

    fn render_frame(frame: &mut Frame, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(frame.size());
        let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = app.input.visual_scroll(width as usize);
        let input = Paragraph::new(app.input.value())
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(app.prompt.clone())
                    .style(Style::default().fg(Color::Reset)),
            )
            .fg(Color::Yellow);
        frame.render_widget(input, chunks[0]);

        let list = List::new(
            app.items
                .iter()
                .map(|item| TuiSearchItem {
                    title: item.title.clone().unwrap_or("".to_string()),
                    subtitle: item.subtitle.clone().unwrap_or("".to_string()),
                    title_style: Style::default().bold(),
                    subtitle_style: Style::default().fg(Color::Gray).italic(),
                })
                .collect(),
        );
        let list_scroll = Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(None)
            .thumb_symbol("▐");
        let list_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(chunks[1]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded);
        frame.render_stateful_widget(list, list_chunks[0], &mut app.list);
        frame.render_stateful_widget(list_scroll, list_chunks[1], &mut app.scroll_state);
        frame.render_widget(block, chunks[1]);
        frame.set_cursor((1 + app.input.visual_cursor()) as u16, 1);
    }

    fn handle_events(app: &mut App) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Esc => {
                        app.os.deinit();
                        app.exit();
                        std::process::exit(0);
                    }
                    event::KeyCode::Enter => {
                        app.exit();
                        let item = app.items.get(app.list.selected().unwrap()).unwrap();
                        if app.os.select(item) {
                            app.os.deinit();
                            std::process::exit(0);
                        } else {
                            app.input.reset();
                            app.items.clear();
                        }
                    }
                    event::KeyCode::Down => {
                        app.list.next();
                        app.scroll_state.next();
                    }
                    event::KeyCode::Up => {
                        app.list.previous();
                        app.scroll_state.prev();
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key_event));
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
    title_style: Style,
    subtitle_style: Style,
}

impl ListableWidget for TuiSearchItem {
    fn size(&self, _scroll_direction: &tui_widget_list::ScrollAxis) -> usize {
        3
    }
    fn highlight(self) -> Self
    where
        Self: Sized,
    {
        Self {
            title_style: self.title_style.fg(Color::Yellow),
            subtitle_style: self.subtitle_style.fg(Color::Yellow),
            ..self
        }
    }
}

impl Widget for TuiSearchItem {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::styled(self.title.clone(), self.title_style);
        let subtitle = Line::styled(self.subtitle.clone(), self.subtitle_style);
        buf.set_line(area.x, area.y, &title, area.width);
        buf.set_line(area.x, area.y + 1, &subtitle, area.width);
    }
}

impl UI for RatatuiUI {
    fn run(&mut self) {
        self.app.search();
        loop {
            self.tui
                .draw(|frame| Self::render_frame(frame, &mut self.app))
                .unwrap();
            Self::handle_events(&mut self.app).unwrap();
        }
    }
}
