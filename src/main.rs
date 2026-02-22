mod dbus;

use dbus::device::{ButtonAction, MouseDevice};
use zbus::Connection;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(PartialEq)]
enum Panel {
    Dpi,
    Buttons,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    EditingButton,
}

struct App {
    device: MouseDevice,
    panel: Panel,
    mode: Mode,
    dpi_state: ListState,
    button_state: ListState,
    // Popup state when editing a button
    popup_state: ListState,
    popup_options: Vec<ButtonAction>,
    status: Option<String>,
}

impl App {
    fn new(device: MouseDevice) -> Self {
        let dpi_index = device
            .valid_dpis
            .iter()
            .position(|&d| d == device.dpi)
            .unwrap_or(0);

        let mut dpi_state = ListState::default();
        dpi_state.select(Some(dpi_index));

        let mut button_state = ListState::default();
        button_state.select(Some(0));

        let mut popup_state = ListState::default();
        popup_state.select(Some(0));

        App {
            device,
            panel: Panel::Dpi,
            mode: Mode::Normal,
            dpi_state,
            button_state,
            popup_state,
            popup_options: Vec::new(),
            status: None,
        }
    }

    fn next_dpi(&mut self) {
        let i = self.dpi_state.selected().unwrap_or(0);
        if i + 1 < self.device.valid_dpis.len() {
            self.dpi_state.select(Some(i + 1));
        }
    }

    fn prev_dpi(&mut self) {
        let i = self.dpi_state.selected().unwrap_or(0);
        if i > 0 {
            self.dpi_state.select(Some(i - 1));
        }
    }

    fn next_button(&mut self) {
        let i = self.button_state.selected().unwrap_or(0);
        if i + 1 < self.device.buttons.len() {
            self.button_state.select(Some(i + 1));
        }
    }

    fn prev_button(&mut self) {
        let i = self.button_state.selected().unwrap_or(0);
        if i > 0 {
            self.button_state.select(Some(i - 1));
        }
    }

    fn next_popup(&mut self) {
        let i = self.popup_state.selected().unwrap_or(0);
        if i + 1 < self.popup_options.len() {
            self.popup_state.select(Some(i + 1));
        }
    }

    fn prev_popup(&mut self) {
        let i = self.popup_state.selected().unwrap_or(0);
        if i > 0 {
            self.popup_state.select(Some(i - 1));
        }
    }

    fn selected_dpi(&self) -> u32 {
        self.dpi_state
            .selected()
            .map(|i| self.device.valid_dpis[i])
            .unwrap_or(self.device.dpi)
    }

fn open_button_editor(&mut self) {
    let mut options = vec![ButtonAction::None];
    for n in 1u32..=8 {
        options.push(ButtonAction::Button(n));
    }

    let current = self.button_state.selected().unwrap_or(0);
    let current_action = &self.device.buttons[current].action;
    let selected = options
        .iter()
        .position(|o| match (o, current_action) {
            (ButtonAction::None, ButtonAction::None) => true,
            (ButtonAction::Button(a), ButtonAction::Button(b)) => a == b,
            _ => false,
        })
        .unwrap_or(0);

    self.popup_options = options;
    self.popup_state.select(Some(selected));
    self.mode = Mode::EditingButton;
}
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ratbagtui",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  ·  "),
        Span::styled(&app.device.name, Style::default().fg(Color::White)),
        Span::raw(format!("  ·  {}dpi", app.device.dpi)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, root[0]);

    // Two panels
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(root[1]);

    // DPI panel
    let dpi_items: Vec<ListItem> = app
        .device
        .valid_dpis
        .iter()
        .map(|&d| {
            let label = if d == app.device.dpi {
                format!("{} dpi  ←", d)
            } else {
                format!("{} dpi", d)
            };
            ListItem::new(label)
        })
        .collect();

    let dpi_style = if app.panel == Panel::Dpi && app.mode == Mode::Normal {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let dpi_list = List::new(dpi_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(dpi_style)
                .title(" DPI "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(dpi_list, panels[0], &mut app.dpi_state);

    // Button panel
    let button_items: Vec<ListItem> = app
        .device
        .buttons
        .iter()
        .map(|btn| {
            ListItem::new(format!(
                "Button {}   {}",
                btn.index,
                btn.action.label()
            ))
        })
        .collect();

    let btn_style = if app.panel == Panel::Buttons && app.mode == Mode::Normal {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let button_list = List::new(button_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(btn_style)
                .title(" Buttons "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(button_list, panels[1], &mut app.button_state);

    // Footer
    let footer_text = if let Some(ref msg) = app.status {
        Line::from(Span::styled(msg, Style::default().fg(Color::Yellow)))
    } else {
        Line::from(vec![
            Span::styled(" Tab ", Style::default().bg(Color::DarkGray)),
            Span::raw(" switch panel  "),
            Span::styled(" ↑↓ ", Style::default().bg(Color::DarkGray)),
            Span::raw(" navigate  "),
            Span::styled(" Enter ", Style::default().bg(Color::DarkGray)),
            Span::raw(" apply  "),
            Span::styled(" q ", Style::default().bg(Color::DarkGray)),
            Span::raw(" quit"),
        ])
    };
    frame.render_widget(Paragraph::new(footer_text), root[2]);

    // Button edit popup
    if app.mode == Mode::EditingButton {
        let popup_area = centered_rect(36, (app.popup_options.len() as u16) + 4, area);
        frame.render_widget(Clear, popup_area);

        let popup_items: Vec<ListItem> = app
            .popup_options
            .iter()
            .map(|a| ListItem::new(a.label()))
            .collect();

        let popup_list = List::new(popup_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(" Select Action ")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(popup_list, popup_area, &mut app.popup_state);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let devices = MouseDevice::load(&conn).await?;

    if devices.is_empty() {
        eprintln!("No devices found. Is ratbagd running?");
        return Ok(());
    }

    let device = devices.into_iter().next().unwrap();
    let mut app = App::new(device);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Tab => {
                        app.panel = if app.panel == Panel::Dpi {
                            Panel::Buttons
                        } else {
                            Panel::Dpi
                        };
                        app.status = None;
                    }

                    KeyCode::Down | KeyCode::Char('j') => match app.panel {
                        Panel::Dpi => app.next_dpi(),
                        Panel::Buttons => app.next_button(),
                    },

                    KeyCode::Up | KeyCode::Char('k') => match app.panel {
                        Panel::Dpi => app.prev_dpi(),
                        Panel::Buttons => app.prev_button(),
                    },

                    KeyCode::Enter => match app.panel {
                        Panel::Dpi => {
                            let new_dpi = app.selected_dpi();
                            if new_dpi != app.device.dpi {
                                match app.device.set_dpi(&conn, new_dpi).await {
                                    Ok(_) => {
                                        app.status = Some(format!("DPI set to {}", new_dpi));
                                    }
                                    Err(e) => {
                                        app.status = Some(format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                        Panel::Buttons => {
                            app.open_button_editor();
                        }
                    },

                    _ => {}
                },

                Mode::EditingButton => match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::Normal;
                    }

                    KeyCode::Down | KeyCode::Char('j') => app.next_popup(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev_popup(),

                    KeyCode::Enter => {
                        let button_index = app.button_state.selected().unwrap_or(0);
                        let action_index = app.popup_state.selected().unwrap_or(0);
                        let action = app.popup_options[action_index].clone();
                        let label = action.label();

                        match app.device.set_button(&conn, button_index, action).await {
                            Ok(_) => {
                                app.status =
                                    Some(format!("Button {} set to {}", button_index, label));
                            }
                            Err(e) => {
                                app.status = Some(format!("Error: {}", e));
                            }
                        }
                        app.mode = Mode::Normal;
                    }

                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}