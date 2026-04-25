use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, BorderType},
    Terminal,
};
use std::{fs, io::{self, BufRead, Read}, path::{Path, PathBuf}, process::Command, env};

const C_PINK: Color = Color::Rgb(242, 200, 248);
const C_DARK_PURPLE: Color = Color::Rgb(106, 49, 143);
const C_AQUA: Color = Color::Rgb(156, 234, 216);
const C_LAVENDER: Color = Color::Rgb(131, 135, 234);
const C_VIBRANT: Color = Color::Rgb(133, 64, 181);
const C_BORDER: Color = Color::Rgb(70, 70, 95); 

#[derive(PartialEq)]
enum Mode { Normal, Command }

struct App {
    items: Vec<String>,
    state: ListState,
    path: PathBuf,
    preview: Vec<String>,
    show_help: bool,
    show_hidden: bool,
    mode: Mode,
    cmd_buffer: String,
    preview_width: usize,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            items: Vec::new(),
            state: ListState::default(),
            path: env::current_dir().unwrap_or_else(|_| PathBuf::from("/home")),
            preview: Vec::new(),
            show_help: false,
            show_hidden: false,
            mode: Mode::Normal,
            cmd_buffer: String::new(),
            preview_width: 0,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.path) {
            let mut list: Vec<String> = entries
                .filter_map(|e| {
                    let entry = e.ok()?;
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if !self.show_hidden && name.starts_with('.') { return None; }
                    Some(name)
                })
                .collect();
            list.sort_by_key(|a| a.to_lowercase());
            let mut final_list = if self.path.parent().is_some() { vec!["..".into()] } else { vec![] };
            final_list.append(&mut list);
            self.items = final_list;
        }
        self.update_preview();
    }

    fn push_clean_line(&mut self, text: String) {
        let clean: String = text.chars().filter(|c| !c.is_control() || c.is_whitespace()).collect();
        let max_w = self.preview_width.saturating_sub(4);
        let truncated: String = clean.chars().take(max_w).collect();
        let padding = " ".repeat(max_w.saturating_sub(truncated.chars().count()));
        self.preview.push(format!(" {}{}", truncated, padding));
    }

    fn is_binary(path: &Path) -> bool {
        if let Ok(mut file) = fs::File::open(path) {
            let mut buffer = [0u8; 1024];
            if let Ok(n) = file.read(&mut buffer) {
                return buffer[..n].iter().any(|&b| b == 0);
            }
        }
        false
    }

    fn update_preview(&mut self) {
        self.preview.clear();
        let selected = match self.state.selected() {
            Some(i) if i < self.items.len() => &self.items[i],
            _ => return,
        };
        
        let full_path = self.path.join(selected);
        
        if selected == ".." {
            self.push_clean_line(" Вернуться назад".into());
        } else if full_path.is_dir() {
            self.push_clean_line(format!(" Папка: {}", selected));
            self.push_clean_line("──────────────────────────".into());
            if let Ok(entries) = fs::read_dir(&full_path) {
                for entry in entries.take(30) { 
                    if let Ok(e) = entry {
                        let icon = if e.path().is_dir() { "" } else { "󰈔" };
                        self.push_clean_line(format!("  {} {}", icon, e.file_name().to_string_lossy()));
                    }
                }
            }
        } else {
            if Self::is_binary(&full_path) {
                self.push_clean_line(" [!] Binary file".into());
                return;
            }

            if let Ok(file) = fs::File::open(&full_path) {
                let reader = io::BufReader::new(file);
                for line in reader.lines().take(40) {
                    if let Ok(l) = line { self.push_clean_line(l); }
                }
            }
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
            let size = f.size();
            let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(3), Constraint::Length(1)]).split(size);
            let main_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(40), Constraint::Percentage(60)]).split(chunks[0]);
            app.preview_width = main_chunks[1].width as usize;

            let items: Vec<ListItem> = app.items.iter().map(|name| {
                let is_dir = name == ".." || app.path.join(name).is_dir();
                ListItem::new(format!(" {}  {}", if is_dir { "" } else { "󰈔" }, name))
                    .style(Style::default().fg(if is_dir { C_AQUA } else { C_PINK }))
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(C_BORDER))
                    .title(format!("  {} ", app.path.display())).title_style(Style::default().fg(C_LAVENDER).add_modifier(Modifier::BOLD)))
                .highlight_style(Style::default().bg(C_DARK_PURPLE).fg(C_AQUA).add_modifier(Modifier::BOLD))
                .highlight_symbol(" ❱ ");
            f.render_stateful_widget(list, main_chunks[0], &mut app.state);

            f.render_widget(Clear, main_chunks[1]);
            let prev = Paragraph::new(app.preview.join("\n"))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(C_BORDER))
                    .title(" 󰈈 Preview ").title_style(Style::default().fg(C_VIBRANT).add_modifier(Modifier::BOLD)));
            f.render_widget(prev, main_chunks[1]);

            // --- ОБНОВЛЕННЫЙ СТАТУСБАР ---
            let bar_content = if app.mode == Mode::Command { 
                format!(" :{}", app.cmd_buffer) 
            } else { 
                format!("  [Use :binds to see keybinds] | Hidden: {}", if app.show_hidden { "ON" } else { "OFF" }) 
            };
            
            let bar_style = if app.mode == Mode::Command {
                Style::default().bg(C_AQUA).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(C_LAVENDER).fg(C_DARK_PURPLE).add_modifier(Modifier::BOLD)
            };
            f.render_widget(Paragraph::new(bar_content).style(bar_style), chunks[1]);

            // --- ОКНО ПОМОЩИ ---
            if app.show_help {
                let area = Rect::new((size.width - 40)/2, (size.height - 10)/2, 40, 10);
                f.render_widget(Clear, area);
                let help_items = vec![
                    ListItem::new(" :o      - Open in nvim"),
                    ListItem::new(" :q      - Quit"),
                    ListItem::new(" :h      - Toggle hidden"),
                    ListItem::new(" :binds  - This menu"),
                    ListItem::new(" ESC     - Close menu"),
                ];
                f.render_widget(List::new(help_items).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" COMMANDS ").border_style(Style::default().fg(C_AQUA))), area);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char(':') => { app.show_help = false; app.mode = Mode::Command; },
                    KeyCode::Char('j') | KeyCode::Down => {
                        let i = match app.state.selected() { Some(i) => if i >= app.items.len()-1 {0} else {i+1}, None => 0 };
                        app.state.select(Some(i)); app.update_preview();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let i = match app.state.selected() { Some(i) => if i == 0 {app.items.len()-1} else {i-1}, None => 0 };
                        app.state.select(Some(i)); app.update_preview();
                    }
                    KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                        if let Some(i) = app.state.selected() {
                            let n = app.items[i].clone();
                            let t = if n == ".." { app.path.parent().unwrap_or(&app.path).to_path_buf() } else { app.path.join(n) };
                            if t.is_dir() { app.path = t; app.refresh(); app.state.select(Some(0)); }
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('h') {
                            app.show_hidden = !app.show_hidden; app.refresh();
                        } else if !key.modifiers.contains(KeyModifiers::CONTROL) {
                            if let Some(p) = app.path.parent() { app.path = p.to_path_buf(); app.refresh(); app.state.select(Some(0)); }
                        }
                    }
                    KeyCode::Esc => app.show_help = false,
                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Enter => {
                        let cmd = app.cmd_buffer.trim();
                        match cmd {
                            "q" => break,
                            "binds" => app.show_help = true,
                            "h" => { app.show_hidden = !app.show_hidden; app.refresh(); },
                            "o" => {
                                if let Some(i) = app.state.selected() {
                                    let t = app.path.join(&app.items[i]);
                                    if t.is_file() {
                                        let _ = disable_raw_mode();
                                        let _ = execute!(io::stdout(), LeaveAlternateScreen);
                                        let _ = Command::new("nvim").arg(t).status();
                                        let _ = enable_raw_mode();
                                        let _ = execute!(io::stdout(), EnterAlternateScreen);
                                        terminal.clear()?; app.refresh();
                                    }
                                }
                            }
                            _ => {}
                        }
                        app.mode = Mode::Normal; app.cmd_buffer.clear();
                    }
                    KeyCode::Esc => { app.mode = Mode::Normal; app.cmd_buffer.clear(); },
                    KeyCode::Backspace => { app.cmd_buffer.pop(); if app.cmd_buffer.is_empty() { app.mode = Mode::Normal; } },
                    KeyCode::Char(c) => app.cmd_buffer.push(c),
                    _ => {}
                }
            }
        }
    }
    disable_raw_mode()?; execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
