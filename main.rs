use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::{fs, io, env, path::PathBuf};

// Настоящий пацанский Gruvbox
const BG: Color = Color::Rgb(40, 40, 40);
const FG: Color = Color::Rgb(235, 219, 178);
const YELLOW: Color = Color::Rgb(215, 153, 33);
const BLUE: Color = Color::Rgb(69, 133, 136);
const GRAY: Color = Color::Rgb(146, 131, 116);

struct App {
    items: Vec<String>,
    state: ListState,
    path: PathBuf,
    preview: String,
}

impl App {
    fn new() -> Self {
        let path = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut app = Self {
            items: Vec::new(),
            state: ListState::default(),
            path,
            preview: String::new(),
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.items = fs::read_dir(&self.path)
            .map(|res| res.filter_map(|e| e.ok().map(|entry| entry.file_name().into_string().unwrap())).collect())
            .unwrap_or_default();
        self.items.sort_by_key(|a| a.to_lowercase());
        
        if !self.items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
        self.update_preview();
    }

    fn update_preview(&mut self) {
        let selected = match self.state.selected() {
            Some(i) => &self.items[i],
            None => { self.preview = "Empty folder".into(); return; }
        };

        let full_path = self.path.join(selected);
        if full_path.is_dir() {
            self.preview = format!("Directory: {}\n\nPress L or Enter to open", selected);
        } else {
            // Читаем только кусок, чтобы не вешать терминал
            self.preview = fs::read_to_string(&full_path)
                .map(|s| s.lines().take(30).collect::<Vec<_>>().join("\n"))
                .unwrap_or_else(|_| "[Binary file or access denied]".into());
        }
    }

    fn enter(&mut self) {
        if let Some(i) = self.state.selected() {
            let next = self.path.join(&self.items[i]);
            if next.is_dir() {
                self.path = next;
                self.refresh();
            }
        }
    }

    fn back(&mut self) {
        if let Some(parent) = self.path.parent() {
            self.path = parent.to_path_buf();
            self.refresh();
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25), // Path
                    Constraint::Percentage(35), // Files
                    Constraint::Percentage(40), // Preview
                ])
                .split(f.size());

            // 1. Панель пути
            let path_box = Paragraph::new(app.path.to_str().unwrap_or("?"))
                .block(Block::default().borders(Borders::ALL).title(" Path ").border_style(Style::default().fg(BLUE)))
                .style(Style::default().bg(BG).fg(FG));
            f.render_widget(path_box, chunks[0]);

            // 2. Список файлов
            let items: Vec<ListItem> = app.items.iter().map(|name| {
                let style = if app.path.join(name).is_dir() {
                    Style::default().fg(BLUE).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(FG)
                };
                ListItem::new(name.as_str()).style(style)
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Justfiles ").border_style(Style::default().fg(YELLOW)))
                .highlight_style(Style::default().bg(Color::Rgb(60, 56, 54)).fg(YELLOW))
                .highlight_symbol("> ");
            f.render_stateful_widget(list, chunks[1], &mut app.state);

            // 3. Превью
            let prev_box = Paragraph::new(app.preview.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Preview ").border_style(Style::default().fg(GRAY)))
                .style(Style::default().bg(BG).fg(FG));
            f.render_widget(prev_box, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    let i = match app.state.selected() {
                        Some(i) => if i >= app.items.len() - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    app.state.select(Some(i));
                    app.update_preview();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let i = match app.state.selected() {
                        Some(i) => if i == 0 { app.items.len() - 1 } else { i - 1 },
                        None => 0,
                    };
                    app.state.select(Some(i));
                    app.update_preview();
                }
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter(),
                KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => app.back(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}