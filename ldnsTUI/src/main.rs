/*!
DNS Server TUI - A terminal UI to control the DNS server

This TUI demonstrates:
- Event loop architecture
- State management
- Terminal rendering with ratatui
- Keyboard input handling
- HTTP API integration
*/

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::io;

// ===== DATA STRUCTURES =====

/// Preset blocklist configuration
///
/// Teaching moment: Separating data from presentation
/// This struct is pure data - no UI logic
#[derive(Debug, Clone)]
struct Preset {
    name: String,
    description: String,
    url: String,
    size: PresetSize,
}

/// Size category for presets
///
/// Teaching moment: Enums can carry semantic meaning
/// Helps users choose based on their needs
#[derive(Debug, Clone, PartialEq)]
enum PresetSize {
    Light,    // ~50k domains, less aggressive
    Standard, // ~150k domains, balanced
    Ultimate, // ~230k domains, most comprehensive
}

impl Preset {
    /// Create preset blocklists
    ///
    /// Teaching moment: Using a constructor function to encapsulate configuration
    /// All presets defined in one place - easy to maintain!
    fn get_presets() -> Vec<Preset> {
        vec![
            Preset {
                name: "Light".to_string(),
                description: "Basic protection - Fast, minimal blocking (~50k domains)".to_string(),
                url: "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/light.txt".to_string(),
                size: PresetSize::Light,
            },
            Preset {
                name: "Standard".to_string(),
                description: "Balanced protection - Good for daily use (~150k domains)".to_string(),
                url: "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/pro.txt".to_string(),
                size: PresetSize::Standard,
            },
            Preset {
                name: "Ultimate".to_string(),
                description: "Maximum protection - Aggressive blocking (~230k domains)".to_string(),
                url: "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/ultimate.txt".to_string(),
                size: PresetSize::Ultimate,
            },
        ]
    }
}

/// Main application state
///
/// Teaching moment: This struct holds ALL the state of our TUI
/// Think of it as a mini-database that gets updated as the user interacts
#[derive(Debug)]
struct App {
    /// Current screen being displayed
    screen: Screen,

    /// URL of the DNS API
    api_url: String,

    /// Current blocklist URL
    current_blocklist_url: String,

    /// Number of domains in blocklist
    domains_count: usize,

    /// Input buffer when user is typing
    input: String,

    /// Status messages to show user
    status_message: String,

    /// Whether we're in input mode
    input_mode: bool,

    /// Available preset blocklists
    presets: Vec<Preset>,

    /// Currently selected preset index in the list
    /// Teaching moment: Option<usize> means "might not have selection"
    /// None = no selection, Some(i) = preset i is selected
    selected_preset_index: usize,

    /// Which preset is currently active (if any)
    active_preset_name: Option<String>,
}

/// Screens in our TUI
///
/// Teaching moment: Using an enum for different "pages" in the UI
/// This is like routing in a web app, but for TUI!
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Home,
    EditBlocklist,
    SelectPreset,  // ← New screen for choosing presets!
}

// ===== API TYPES =====

#[derive(Serialize)]
struct BlocklistUpdateRequest {
    url: String,
}

#[derive(Deserialize)]
struct BlocklistUpdateResponse {
    message: String,
    url: String,
    domains_loaded: usize,
}

// ===== IMPLEMENTATION =====

impl App {
    fn new(api_url: String) -> Self {
        let presets = Preset::get_presets();
        let default_url = presets[2].url.clone(); // Start with Ultimate

        Self {
            screen: Screen::Home,
            api_url,
            current_blocklist_url: default_url,
            domains_count: 0,
            input: String::new(),
            status_message: "Ready - Press [P] to select preset blocklist".to_string(),
            input_mode: false,
            presets,
            selected_preset_index: 2, // Start with Ultimate selected
            active_preset_name: Some("Ultimate".to_string()),
        }
    }

    /// Handle keyboard input
    ///
    /// Teaching moment: This is where we translate keypresses into actions
    /// Different keys do different things based on current state (state machine!)
    fn handle_key(&mut self, key: KeyCode) {
        // Teaching moment: Match on screen first, then handle keys differently per screen
        match self.screen {
            Screen::SelectPreset => {
                // Preset selection screen - arrow keys to navigate, Enter to select
                match key {
                    KeyCode::Up => {
                        // Move selection up (with wrapping)
                        if self.selected_preset_index > 0 {
                            self.selected_preset_index -= 1;
                        } else {
                            // Wrap to bottom
                            self.selected_preset_index = self.presets.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        // Move selection down (with wrapping)
                        if self.selected_preset_index < self.presets.len() - 1 {
                            self.selected_preset_index += 1;
                        } else {
                            // Wrap to top
                            self.selected_preset_index = 0;
                        }
                    }
                    KeyCode::Enter => {
                        // Select the current preset
                        self.select_current_preset();
                    }
                    KeyCode::Esc => {
                        // Cancel selection, go back to home
                        self.screen = Screen::Home;
                    }
                    _ => {}
                }
            }
            _ => {
                // Home screen or edit screen
                if self.input_mode {
                    // User is typing input
                    match key {
                        KeyCode::Enter => {
                            // Submit the input
                            self.submit_input();
                        }
                        KeyCode::Esc => {
                            // Cancel input
                            self.input_mode = false;
                            self.input.clear();
                            self.screen = Screen::Home;
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        _ => {}
                    }
                } else {
                    // Normal navigation mode
                    match key {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            // Quit will be handled in main loop
                        }
                        KeyCode::Char('b') | KeyCode::Char('B') => {
                            // Start editing blocklist URL (manual entry)
                            self.screen = Screen::EditBlocklist;
                            self.input_mode = true;
                            self.input = self.current_blocklist_url.clone();
                        }
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            // Open preset selection screen
                            self.screen = Screen::SelectPreset;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Select the currently highlighted preset
    ///
    /// Teaching moment: Separating selection logic from key handling
    /// Makes code easier to test and understand
    fn select_current_preset(&mut self) {
        let preset = &self.presets[self.selected_preset_index];
        self.current_blocklist_url = preset.url.clone();
        self.active_preset_name = Some(preset.name.clone());
        self.status_message = format!("Switching to {} preset...", preset.name);
        self.screen = Screen::Home;
    }

    /// Submit the input (e.g., new blocklist URL)
    fn submit_input(&mut self) {
        if self.screen == Screen::EditBlocklist {
            let new_url = self.input.clone();
            self.status_message = format!("Updating blocklist to custom URL...");

            // Manual URL entry - clear active preset
            self.current_blocklist_url = new_url;
            self.active_preset_name = None;  // ← Custom URL, not a preset
        }

        self.input_mode = false;
        self.input.clear();
        self.screen = Screen::Home;
    }

    /// Update blocklist via API (async)
    async fn update_blocklist(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/blocklist", self.api_url);

        self.status_message = "Sending request...".to_string();

        let response = client
            .put(&url)
            .json(&BlocklistUpdateRequest {
                url: self.current_blocklist_url.clone(),
            })
            .send()
            .await?;

        if response.status().is_success() {
            let data: BlocklistUpdateResponse = response.json().await?;
            self.domains_count = data.domains_loaded;
            self.status_message = format!("✓ {}", data.message);
        } else {
            self.status_message = format!("✗ Error: {}", response.status());
        }

        Ok(())
    }
}

// ===== UI RENDERING =====

/// Draw the UI
///
/// Teaching moment: This function is called every frame (like a game loop!)
/// It rebuilds the entire UI from scratch each time
fn draw_ui(f: &mut ratatui::Frame, app: &App) {
    // Create the main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("DNS Server Control Panel")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content - changes based on screen
    match app.screen {
        Screen::Home => draw_home_screen(f, app, chunks[1]),
        Screen::EditBlocklist => draw_edit_screen(f, app, chunks[1]),
        Screen::SelectPreset => draw_preset_selection_screen(f, app, chunks[1]),
    }

    // Status bar - changes based on context
    let status_text = match app.screen {
        Screen::SelectPreset => {
            // Different controls for preset selection
            vec![
                Line::from(vec![
                    Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" Navigate  "),
                    Span::styled("Enter", Style::default().fg(Color::Green)),
                    Span::raw(" Select  "),
                    Span::styled("Esc", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]),
            ]
        }
        _ => {
            if app.input_mode {
                vec![
                    Line::from(vec![
                        Span::styled("Editing", Style::default().fg(Color::Yellow)),
                        Span::raw(" | "),
                        Span::raw("Press "),
                        Span::styled("Enter", Style::default().fg(Color::Green)),
                        Span::raw(" to submit, "),
                        Span::styled("Esc", Style::default().fg(Color::Red)),
                        Span::raw(" to cancel"),
                    ]),
                ]
            } else {
                vec![
                    Line::from(vec![
                        Span::styled("[P]", Style::default().fg(Color::Cyan)),
                        Span::raw(" Presets  "),
                        Span::styled("[B]", Style::default().fg(Color::Green)),
                        Span::raw(" Custom URL  "),
                        Span::styled("[Q]", Style::default().fg(Color::Red)),
                        Span::raw(" Quit"),
                    ]),
                    Line::from(vec![
                        Span::raw("Status: "),
                        Span::styled(&app.status_message, Style::default().fg(Color::Yellow)),
                    ]),
                ]
            }
        }
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(status, chunks[2]);
}

/// Draw the home screen
fn draw_home_screen(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let info_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Preset: ", Style::default().add_modifier(Modifier::BOLD)),
            match &app.active_preset_name {
                Some(name) => Span::styled(name, Style::default().fg(Color::Cyan)),
                None => Span::styled("Custom URL", Style::default().fg(Color::Yellow)),
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Current Blocklist:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::raw(&app.current_blocklist_url)),
        Line::from(""),
        Line::from(vec![
            Span::styled("Domains Blocked: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                app.domains_count.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("API Endpoint: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&app.api_url),
        ]),
    ];

    let paragraph = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw the edit screen (when entering new blocklist URL)
fn draw_edit_screen(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let input_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter new blocklist URL:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(&app.input, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(vec![
            Span::raw("Cursor: "),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Edit Blocklist URL"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Draw the preset selection screen
///
/// Teaching moment: List navigation pattern
/// - Use arrow keys to move selection
/// - Highlight current selection with different colors
/// - Show details about each option
fn draw_preset_selection_screen(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Choose a preset blocklist:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    // Draw each preset
    for (index, preset) in app.presets.iter().enumerate() {
        let is_selected = index == app.selected_preset_index;
        let is_active = app.active_preset_name.as_ref() == Some(&preset.name);

        // Selection indicator
        let indicator = if is_selected {
            Span::styled("► ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::raw("  ")
        };

        // Preset name with color coding
        let name_style = if is_active {
            // Active preset - green
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else if is_selected {
            // Selected but not active - cyan
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            // Not selected - default
            Style::default()
        };

        let status_marker = if is_active {
            Span::styled(" ✓", Style::default().fg(Color::Green))
        } else {
            Span::raw("")
        };

        // Preset name line
        lines.push(Line::from(vec![
            indicator,
            Span::styled(&preset.name, name_style),
            status_marker,
        ]));

        // Description (only for selected item)
        if is_selected {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(&preset.description, Style::default().fg(Color::Gray)),
            ]));
        }

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Select Preset"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

// ===== MAIN PROGRAM =====

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API URL from environment or use default
    let api_url = std::env::var("DNS_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Initialize the app state
    let mut app = App::new(api_url);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

/// Main event loop
///
/// Teaching moment: This is the heart of the TUI!
/// 1. Draw the UI
/// 2. Check for keyboard input (with timeout)
/// 3. Update state based on input
/// 4. Repeat!
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Draw the current state
        terminal.draw(|f| draw_ui(f, app))?;

        // Wait for input (with timeout so we can update periodically)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') if !app.input_mode => {
                        // Quit the application
                        return Ok(());
                    }
                    code => {
                        app.handle_key(code);

                        // If user just submitted a new URL, update it
                        if !app.input_mode && app.screen == Screen::Home {
                            if let Err(e) = app.update_blocklist().await {
                                app.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}
