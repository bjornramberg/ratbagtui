mod dbus;

use dbus::device::MouseDevice;
use zbus::Connection;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

#[derive(PartialEq)]
enum Panel {
    Dpi,
    Buttons,
}

struct App {
    device: MouseDevice,
    panel: Panel,
    dpi_state: ListState,
    button_state: ListState,
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

        App {
            device,
            panel: Panel::Dpi,
            dpi_state,
            button_state,
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

    fn selected_dpi(&self) -> u32 {
        self.dpi_state
            .selected()
            .map(|i| self.device.valid_dpis[i])
            .unwrap_or(self.device.dpi)
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Header + main split
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled("ratbagtui", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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

    let dpi_style = if app.panel == Panel::Dpi {
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
        .map(|btn| ListItem::new(format!("Button {}   {}", btn.index, btn.action.label())))
        .collect();

    let btn_style = if app.panel == Panel::Buttons {
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
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Tab ", Style::default().bg(Color::DarkGray)),
        Span::raw(" switch panel  "),
        Span::styled(" ↑↓ ", Style::default().bg(Color::DarkGray)),
        Span::raw(" navigate  "),
        Span::styled(" Enter ", Style::default().bg(Color::DarkGray)),
        Span::raw(" apply  "),
        Span::styled(" q ", Style::default().bg(Color::DarkGray)),
        Span::raw(" quit"),
    ]));
    frame.render_widget(footer, root[2]);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let devices = MouseDevice::load(&conn).await?;

    if devices.is_empty() {
        eprintln!("No devices found. Is ratbagd running?");
        return Ok(());
    }

    // Use first device for now
    let device = devices.into_iter().next().unwrap();
    let mut app = App::new(device);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,

                KeyCode::Tab => {
                    app.panel = if app.panel == Panel::Dpi {
                        Panel::Buttons
                    } else {
                        Panel::Dpi
                    };
                }

                KeyCode::Down | KeyCode::Char('j') => match app.panel {
                    Panel::Dpi => app.next_dpi(),
                    Panel::Buttons => app.next_button(),
                },

                KeyCode::Up | KeyCode::Char('k') => match app.panel {
                    Panel::Dpi => app.prev_dpi(),
                    Panel::Buttons => app.prev_button(),
                },

                KeyCode::Enter => {
                    if app.panel == Panel::Dpi {
                        let new_dpi = app.selected_dpi();
                        // We'll wire up the actual write in the next step
                        app.device.dpi = new_dpi;
                    }
                }

                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}