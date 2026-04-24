use std::{
    io::IsTerminal,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, StatefulWidget, Table, TableState, Wrap,
    },
};

use crate::{
    core::{
        ListenerRecord, PortDetails, PortService, Scope, build_kill_plan, execute_kill,
        warnings_for_listener,
    },
    error::{PortxError, Result},
};

const TICK_RATE: Duration = Duration::from_secs(1);

pub fn run(service: &PortService) -> Result<()> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(PortxError::TuiRequiresTerminal);
    }

    let mut terminal = ratatui::try_init()?;
    let result = run_app(&mut terminal, service);
    ratatui::try_restore()?;
    result
}

fn run_app(terminal: &mut DefaultTerminal, service: &PortService) -> Result<()> {
    let mut app = App::default();
    app.refresh(service)?;

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        let timeout = TICK_RATE.saturating_sub(app.last_refresh.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.handle_key(key, service)? {
                break;
            }
        }

        if app.last_refresh.elapsed() >= TICK_RATE {
            app.refresh(service)?;
        }
    }

    Ok(())
}

#[derive(Debug)]
struct App {
    records: Vec<ListenerRecord>,
    details: Vec<PortDetails>,
    selected: usize,
    detail_focus: bool,
    detail_scroll: u16,
    show_help: bool,
    kill_prompt: bool,
    status: Option<StatusMessage>,
    last_refresh: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            details: Vec::new(),
            selected: 0,
            detail_focus: false,
            detail_scroll: 0,
            show_help: false,
            kill_prompt: false,
            status: None,
            last_refresh: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
struct StatusMessage {
    text: String,
    level: StatusLevel,
}

#[derive(Debug, Clone, Copy, Default)]
enum StatusLevel {
    #[default]
    Info,
    Success,
    Error,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            text: String::new(),
            level: StatusLevel::Info,
        }
    }
}

impl App {
    fn refresh(&mut self, service: &PortService) -> Result<()> {
        let previous = self.selected_listener_key();
        let records = service.list(None)?;

        self.records = records;
        self.selected = self.resolve_selection(previous);
        self.details = self.current_details(service)?;
        self.last_refresh = Instant::now();
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, service: &PortService) -> Result<bool> {
        if self.kill_prompt {
            return self.handle_kill_prompt(key, service);
        }

        if self.show_help {
            return Ok(self.handle_help_key(key));
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Esc => {
                self.detail_focus = false;
                self.detail_scroll = 0;
            }
            KeyCode::Down => {
                if self.detail_focus {
                    self.scroll_details_down(1);
                } else {
                    self.next(service)?;
                }
            }
            KeyCode::Up => {
                if self.detail_focus {
                    self.scroll_details_up(1);
                } else {
                    self.previous(service)?;
                }
            }
            KeyCode::PageDown => self.scroll_details_down(8),
            KeyCode::PageUp => self.scroll_details_up(8),
            KeyCode::Home => self.detail_scroll = 0,
            KeyCode::Enter => self.detail_focus = !self.detail_focus,
            KeyCode::Char('k') => self.start_kill_prompt(),
            KeyCode::Char('?') | KeyCode::Char('h') => self.show_help = true,
            _ => {}
        }

        Ok(false)
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => true,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') => {
                self.show_help = false;
                false
            }
            _ => false,
        }
    }

    fn handle_kill_prompt(&mut self, key: KeyEvent, service: &PortService) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.kill_prompt = false;
                self.perform_kill(service)?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.kill_prompt = false;
                self.status = Some(StatusMessage {
                    text: "Cancelled kill request".to_string(),
                    level: StatusLevel::Info,
                });
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }

        Ok(false)
    }

    fn next(&mut self, service: &PortService) -> Result<()> {
        if self.records.is_empty() {
            return Ok(());
        }

        self.selected = (self.selected + 1) % self.records.len();
        self.details = self.current_details(service)?;
        self.detail_scroll = 0;
        Ok(())
    }

    fn previous(&mut self, service: &PortService) -> Result<()> {
        if self.records.is_empty() {
            return Ok(());
        }

        self.selected = if self.selected == 0 {
            self.records.len() - 1
        } else {
            self.selected - 1
        };
        self.details = self.current_details(service)?;
        self.detail_scroll = 0;
        Ok(())
    }

    fn start_kill_prompt(&mut self) {
        if self
            .selected_listener()
            .and_then(|record| record.pid)
            .is_some()
        {
            self.kill_prompt = true;
        } else {
            self.status = Some(StatusMessage {
                text: "Selected listener has no killable PID".to_string(),
                level: StatusLevel::Error,
            });
        }
    }

    fn perform_kill(&mut self, service: &PortService) -> Result<()> {
        let Some(record) = self.selected_listener() else {
            return Ok(());
        };

        let plan = build_kill_plan(service, record.port, record.pid, false)?;
        let result = execute_kill(service, plan)?;
        self.status = Some(StatusMessage {
            text: format!(
                "Sent SIGTERM to PID {} ({})",
                result.pid,
                result.process_name.as_deref().unwrap_or("N/A")
            ),
            level: StatusLevel::Success,
        });
        self.refresh(service)?;
        Ok(())
    }

    fn current_details(&self, service: &PortService) -> Result<Vec<PortDetails>> {
        let Some(record) = self.selected_listener() else {
            return Ok(Vec::new());
        };

        service.info(record.port, record.pid)
    }

    fn selected_listener(&self) -> Option<&ListenerRecord> {
        self.records.get(self.selected)
    }

    fn selected_listener_key(&self) -> Option<ListenerKey> {
        self.selected_listener().map(ListenerKey::from)
    }

    fn resolve_selection(&self, previous: Option<ListenerKey>) -> usize {
        if self.records.is_empty() {
            return 0;
        }

        if let Some(previous) = previous
            && let Some(index) = self
                .records
                .iter()
                .position(|record| ListenerKey::from(record) == previous)
        {
            return index;
        }

        self.selected.min(self.records.len() - 1)
    }

    fn scroll_details_down(&mut self, step: u16) {
        self.detail_scroll = self.detail_scroll.saturating_add(step);
    }

    fn scroll_details_up(&mut self, step: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(step);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListenerKey {
    port: u16,
    pid: Option<u32>,
    bind_addr: String,
}

impl From<&ListenerRecord> for ListenerKey {
    fn from(value: &ListenerRecord) -> Self {
        Self {
            port: value.port,
            pid: value.pid,
            bind_addr: value.bind_addr.to_string(),
        }
    }
}

fn render(frame: &mut Frame, app: &App) {
    let outer = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(10),
        Constraint::Length(4),
    ])
    .split(frame.area());

    render_header(frame, outer[0], app);

    if app.detail_focus {
        render_details(frame, outer[1], app, true);
    } else {
        let content = Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)])
            .split(outer[1]);
        render_list(frame, content[0], app);
        render_details(frame, content[1], app, false);
    }

    render_footer(frame, outer[2], app);

    if app.kill_prompt {
        render_kill_prompt(frame, app);
    } else if app.show_help {
        render_help(frame);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app
        .selected_listener()
        .map(selected_summary)
        .unwrap_or_else(|| "No listener selected".to_string());
    let counts = scope_counts(&app.records);
    let mode = if app.detail_focus {
        "Detail focus"
    } else {
        "Split view"
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "portx tui",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Listeners: {}", app.records.len()),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Public: {}  Lan: {}  Local: {}",
                    counts.public, counts.lan, counts.local
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::DarkGray)),
            Span::styled(selected, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(Color::DarkGray)),
            Span::styled(mode, Style::default().fg(Color::Gray)),
            Span::raw("  "),
            Span::styled("Refresh: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ago", format_elapsed(app.last_refresh.elapsed())),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Overview"));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, area: Rect, app: &App) {
    if app.records.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(Span::styled(
                "No listening ports found.",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Start a service or wait for the next refresh."),
        ])
        .block(Block::default().borders(Borders::ALL).title("Listeners"));

        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(["Port", "Scope", "Proto", "PID", "Process", "Bind", "Risk"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows = app.records.iter().map(|record| {
        let warnings = warnings_for_listener(record);
        let risk = warning_badge(&warnings);

        Row::new(vec![
            Cell::from(record.port.to_string()),
            Cell::from(scope_span(record.scope)),
            Cell::from(record.protocol.to_string()),
            Cell::from(
                record
                    .pid
                    .map_or_else(|| "-".to_string(), |pid| pid.to_string()),
            ),
            Cell::from(record.process_name.as_deref().unwrap_or("N/A").to_string()),
            Cell::from(record.bind_addr.to_string()),
            Cell::from(Span::styled(risk.label, risk.style)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(18),
            Constraint::Min(12),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .column_spacing(1)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Listeners ({})", app.records.len())),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ");

    let mut state = TableState::default().with_selected(if app.records.is_empty() {
        None
    } else {
        Some(app.selected)
    });
    StatefulWidget::render(table, area, frame.buffer_mut(), &mut state);
}

fn render_details(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    let title = if focused {
        "Details (focused)"
    } else {
        "Details"
    };

    let body = if app.details.is_empty() {
        vec![Line::from("No listener details available.")]
    } else {
        detail_lines(&app.details)
    };

    let paragraph = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                })
                .title(title),
        )
        .scroll((app.detail_scroll, 0))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let status = app.status.clone().unwrap_or(StatusMessage {
        text: "Ready. Press ? for help.".to_string(),
        level: StatusLevel::Info,
    });

    let style = match status.level {
        StatusLevel::Info => Style::default().fg(Color::Gray),
        StatusLevel::Success => Style::default().fg(Color::Green),
        StatusLevel::Error => Style::default().fg(Color::Red),
    };

    let footer = Paragraph::new(vec![
        Line::from(Span::styled(status.text, style)),
        Line::from(vec![
            Span::styled("Shortcuts: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Up/Down",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.detail_focus {
                    " scroll  "
                } else {
                    " move  "
                },
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" focus  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "PgUp/PgDn",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" details  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "k",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" kill  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "?/h",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" help  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" quit", Style::default().fg(Color::Gray)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn render_kill_prompt(frame: &mut Frame, app: &App) {
    let Some(record) = app.selected_listener() else {
        return;
    };

    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Terminate selected listener?",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Port: {}", record.port)),
        Line::from(format!(
            "PID: {}",
            record
                .pid
                .map_or_else(|| "N/A".to_string(), |pid| pid.to_string())
        )),
        Line::from(format!(
            "Process: {}",
            record.process_name.as_deref().unwrap_or("N/A")
        )),
        Line::from(format!(
            "Command: {}",
            record.command.as_deref().unwrap_or("N/A")
        )),
        Line::from(""),
        Line::from("Press y to confirm, n or Esc to cancel."),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Kill confirmation"),
    )
    .wrap(Wrap { trim: true });

    frame.render_widget(text, area);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(72, 56, frame.area());
    frame.render_widget(Clear, area);

    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Keyboard shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Up / Down  Move the selection in the listener list"),
        Line::from("Enter      Toggle detail focus mode"),
        Line::from("PgUp/PgDn  Scroll details faster"),
        Line::from("Home       Jump to the top of the details pane"),
        Line::from("Esc        Leave detail focus or close this help"),
        Line::from("k          Open the kill confirmation dialog"),
        Line::from("y / n      Confirm or cancel the kill dialog"),
        Line::from("? / h      Open or close this help panel"),
        Line::from("q          Quit the TUI"),
        Line::from(""),
        Line::from(Span::styled(
            "Scope legend",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("PUBLIC     Exposed to external networks"),
        Line::from("LAN        Reachable on local network ranges"),
        Line::from("LOCAL      Bound to loopback only"),
        Line::from(""),
        Line::from(Span::styled(
            "Notes",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("The screen refreshes once per second."),
        Line::from("The selected row drives the details pane on the right."),
        Line::from("When detail focus is on, Up / Down scroll details instead of changing rows."),
    ])
    .block(Block::default().borders(Borders::ALL).title("Help"))
    .wrap(Wrap { trim: true });

    frame.render_widget(text, area);
}

fn centered_rect(horizontal_percent: u16, vertical_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - vertical_percent) / 2),
        Constraint::Percentage(vertical_percent),
        Constraint::Percentage((100 - vertical_percent) / 2),
    ])
    .flex(Flex::Center)
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - horizontal_percent) / 2),
        Constraint::Percentage(horizontal_percent),
        Constraint::Percentage((100 - horizontal_percent) / 2),
    ])
    .flex(Flex::Center)
    .split(vertical[1]);

    horizontal[1]
}

fn detail_lines(details: &[PortDetails]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (index, detail) in details.iter().enumerate() {
        let warnings = warning_badge(&detail.warnings);

        if index > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "----------------------------------------",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            format!(
                "[Listener {}] Port {} / {} / {}",
                index + 1,
                detail.listener.port,
                detail.listener.protocol,
                detail.listener.scope
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(section_label("Network"));
        lines.push(label_value_line(
            "Bind",
            format!("{}", detail.listener.bind_addr),
        ));
        lines.push(label_value_line(
            "Scope",
            format!("{}", detail.listener.scope),
        ));
        lines.push(Line::from(vec![
            Span::styled(
                "Risk",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(warnings.label.to_string(), warnings.style),
        ]));
        lines.push(label_value_line(
            "Connections",
            detail
                .connection_count
                .map_or_else(|| "N/A".to_string(), |connections| connections.to_string()),
        ));
        lines.push(Line::from(""));
        lines.push(section_label("Process"));
        lines.push(label_value_line(
            "Name",
            detail
                .listener
                .process_name
                .as_deref()
                .unwrap_or("N/A")
                .to_string(),
        ));
        lines.push(label_value_line(
            "PID",
            detail
                .listener
                .pid
                .map_or_else(|| "N/A".to_string(), |pid| pid.to_string()),
        ));
        lines.push(label_value_line(
            "User",
            detail.user.as_deref().unwrap_or("N/A").to_string(),
        ));
        lines.push(label_value_line(
            "Command",
            detail
                .listener
                .command
                .as_deref()
                .unwrap_or("N/A")
                .to_string(),
        ));
        lines.push(label_value_line(
            "CWD",
            detail
                .cwd
                .as_ref()
                .map_or_else(|| "N/A".to_string(), |cwd| cwd.display().to_string()),
        ));
        lines.push(Line::from(""));
        lines.push(section_label("Resources"));
        lines.push(label_value_line(
            "CPU",
            detail
                .cpu_percent
                .map_or_else(|| "N/A".to_string(), |cpu| format!("{cpu:.1}%")),
        ));
        lines.push(label_value_line(
            "Memory",
            detail
                .memory_bytes
                .map_or_else(|| "N/A".to_string(), format_bytes),
        ));
        lines.push(label_value_line(
            "Threads",
            detail
                .thread_count
                .map_or_else(|| "N/A".to_string(), |threads| threads.to_string()),
        ));
        lines.push(label_value_line(
            "Uptime",
            detail
                .uptime_seconds
                .map_or_else(|| "N/A".to_string(), format_duration),
        ));
    }

    lines
}

fn selected_summary(record: &ListenerRecord) -> String {
    let warnings = warnings_for_listener(record);
    let risk = warning_badge(&warnings).label;

    format!(
        "{} {} {} {} [{}]",
        record.port,
        record.process_name.as_deref().unwrap_or("N/A"),
        record.bind_addr,
        record.scope,
        risk
    )
}

fn label_value_line(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            label.to_string(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(value, Style::default().fg(Color::White)),
    ])
}

fn section_label(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn scope_span(scope: Scope) -> Span<'static> {
    let (color, label) = match scope {
        Scope::Public => (Color::Red, "PUBLIC"),
        Scope::Lan => (Color::Yellow, "LAN"),
        Scope::Local => (Color::Green, "LOCAL"),
    };

    Span::styled(
        label.to_string(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn format_elapsed(duration: Duration) -> String {
    let seconds = duration.as_secs();

    if seconds == 0 {
        "just now".to_string()
    } else {
        format!("{seconds}s")
    }
}

struct ScopeCounts {
    public: usize,
    lan: usize,
    local: usize,
}

fn scope_counts(records: &[ListenerRecord]) -> ScopeCounts {
    let mut counts = ScopeCounts {
        public: 0,
        lan: 0,
        local: 0,
    };

    for record in records {
        match record.scope {
            Scope::Public => counts.public += 1,
            Scope::Lan => counts.lan += 1,
            Scope::Local => counts.local += 1,
        }
    }

    counts
}

struct WarningBadge {
    label: &'static str,
    style: Style,
}

fn warning_badge<T>(warnings: &[T]) -> WarningBadge
where
    T: ToString,
{
    if warnings.is_empty() {
        WarningBadge {
            label: "OK",
            style: Style::default().fg(Color::Green),
        }
    } else if warnings
        .iter()
        .any(|warning| warning.to_string().contains("wildcard"))
    {
        WarningBadge {
            label: "WILDCARD",
            style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        }
    } else {
        WarningBadge {
            label: "PUBLIC",
            style: Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
