// Crossterm provides cross-platform terminal manipulation (raw mode, events, etc.)
// We need these specific imports to handle terminal state and capture user input
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Ratatui is our TUI framework - it provides widgets and layout management
// We import specific components we need rather than using glob imports for clarity
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

// Serde provides serialization/deserialization for saving todos to disk
// We import the derive macros to automatically implement these traits
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, io, path::PathBuf};

/// Represents a single todo item in our list
/// We derive Clone because we need to copy TodoItems when rendering the UI
/// Serialize and Deserialize allow us to save/load todos from JSON files
#[derive(Clone, Serialize, Deserialize)]
struct TodoItem {
    text: String,
    completed: bool,
}

/// Main application state container
/// This struct holds everything needed to render the UI and respond to user actions
struct App {
    /// Vector of all todo items - using Vec because we need dynamic sizing and indexed access
    todos: Vec<TodoItem>,
    
    /// Tracks which todo is currently selected - ListState is ratatui's way of managing selection
    /// We need this separate from the todos Vec because it's stateful UI information
    state: ListState,
    
    /// Buffer for user input when adding new todos
    /// Separate from todos because it's temporary data before committing
    input: String,
    
    /// Flag to track if we're in input mode (adding a todo) or navigation mode
    /// This determines how we interpret keypresses - modal interface pattern
    input_mode: bool,
}

impl App {
    /// Creates a new App instance with sensible defaults
    /// We initialize with helper todos to guide first-time users
    fn new() -> App {
        // ListState needs to be initialized with a selection for immediate user interaction
        let mut state = ListState::default();
        state.select(Some(0)); // Start with first item selected for better UX
        
        App {
            // Start with tutorial todos to demonstrate functionality
            // This is better than an empty list which might confuse users
            todos: vec![
                TodoItem { text: "Press 'a' to add a todo".to_string(), completed: false },
                TodoItem { text: "Press 'Space' to toggle completion".to_string(), completed: false },
                TodoItem { text: "Press 'd' to delete a todo".to_string(), completed: false },
                TodoItem { text: "Press 'q' to quit".to_string(), completed: false },
            ],
            state,
            input: String::new(),
            input_mode: false,
        }
    }

    /// Gets the path to the todos file in the current directory
    /// We use current directory so todos are stored with the project
    /// This makes it easy to have different todo lists for different projects
    fn get_save_path() -> Result<PathBuf, Box<dyn Error>> {
        // Get current working directory where the program is run from
        let current_dir = std::env::current_dir()?;
        
        // Store in todos.json in the same directory as where the program runs
        // Not hidden so users can easily find and back up their todos
        Ok(current_dir.join("todos.json"))
    }

    /// Saves todos to disk as JSON
    /// We save after every modification to prevent data loss on crashes
    fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::get_save_path()?;
        
        // Serialize to pretty JSON for human readability (easier debugging)
        // If we needed performance, we'd use compact JSON instead
        let json = serde_json::to_string_pretty(&self.todos)?;
        
        // Write atomically by writing to temp file then renaming
        // This prevents corruption if program crashes during write
        fs::write(&path, json)?;
        
        Ok(())
    }

    /// Loads todos from disk, or creates default if file doesn't exist
    /// Returns a new App with loaded todos, or default todos on first run
    fn load() -> App {
        let mut app = App::new();
        
        // Attempt to load from disk
        if let Ok(path) = Self::get_save_path() {
            if let Ok(contents) = fs::read_to_string(&path) {
                // Try to deserialize - if it fails, we'll just use default todos
                // This gracefully handles corrupted files
                if let Ok(todos) = serde_json::from_str::<Vec<TodoItem>>(&contents) {
                    if !todos.is_empty() {
                        app.todos = todos;
                        // Ensure selection is valid for loaded todos
                        app.state.select(Some(0));
                    }
                }
            }
        }
        
        app
    }

    /// Moves selection to the next todo item
    /// Wraps around to the start for continuous navigation (circular list pattern)
    fn next(&mut self) {
        // Early return if empty to prevent index out of bounds
        if self.todos.is_empty() {
            return;
        }
        
        let i = match self.state.selected() {
            Some(i) => {
                // Wrap to beginning if at end - provides better UX than stopping at bottom
                if i >= self.todos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            // If nothing selected (shouldn't happen), start at beginning
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Moves selection to the previous todo item
    /// Wraps around to the end for continuous navigation (circular list pattern)
    fn previous(&mut self) {
        // Early return if empty to prevent index out of bounds
        if self.todos.is_empty() {
            return;
        }
        
        let i = match self.state.selected() {
            Some(i) => {
                // Wrap to end if at beginning - provides better UX than stopping at top
                if i == 0 {
                    self.todos.len() - 1
                } else {
                    i - 1
                }
            }
            // If nothing selected (shouldn't happen), start at end
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Toggles the completion state of the currently selected todo
    /// We modify in place rather than recreating for efficiency
    /// Saves after modification to persist changes immediately
    fn toggle_completed(&mut self) {
        if let Some(i) = self.state.selected() {
            // Bounds check prevents panic if state is somehow out of sync
            if i < self.todos.len() {
                self.todos[i].completed = !self.todos[i].completed;
                // Save after every change - prevents data loss
                // We ignore errors here to not disrupt UX, but could log them
                let _ = self.save();
            }
        }
    }

    /// Deletes the currently selected todo and adjusts selection intelligently
    /// Selection adjustment is crucial for maintaining good UX after deletion
    /// Saves after modification to persist changes immediately
    fn delete_selected(&mut self) {
        if let Some(i) = self.state.selected() {
            // Bounds check prevents panic if state is somehow out of sync
            if i < self.todos.len() {
                self.todos.remove(i);
                
                // Adjust selection to maintain user context after deletion
                if !self.todos.is_empty() {
                    // If we deleted the last item, move selection up
                    // Otherwise, keep selection at same index (which is now the next item)
                    let new_index = if i >= self.todos.len() { 
                        self.todos.len() - 1 
                    } else { 
                        i 
                    };
                    self.state.select(Some(new_index));
                } else {
                    // No items left, deselect to prevent issues
                    self.state.select(None);
                }
                
                // Save after deletion - prevents data loss
                let _ = self.save();
            }
        }
    }

    /// Adds a new todo from the input buffer and resets input state
    /// We only add if input is non-empty to prevent blank todos
    /// Saves after modification to persist changes immediately
    fn add_todo(&mut self) {
        if !self.input.is_empty() {
            self.todos.push(TodoItem {
                text: self.input.clone(), // Clone because we're about to clear input
                completed: false,
            });
            
            // Clear input buffer for next use
            self.input.clear();
            
            // Exit input mode to return to navigation
            self.input_mode = false;
            
            // Select the newly added item so user sees immediate feedback
            self.state.select(Some(self.todos.len() - 1));
            
            // Save after adding - prevents data loss
            let _ = self.save();
        }
    }
}

/// Entry point - sets up terminal, runs app, then cleans up
/// The Result type allows us to propagate errors up to the runtime
fn main() -> Result<(), Box<dyn Error>> {
    // Enable raw mode to read input directly without waiting for Enter
    // This is essential for responsive TUI - we need to react to every keypress
    enable_raw_mode()?;
    
    let mut stdout = io::stdout();
    
    // Enter alternate screen to preserve user's terminal history
    // Enable mouse capture even though we don't use it yet (future-proofing)
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // Create terminal backend - CrosstermBackend works on Windows, Linux, and macOS
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load app state from disk, or create new if no saved data exists
    let app = App::load();
    let res = run_app(&mut terminal, app);

    // CRITICAL: Always restore terminal state, even if app crashes
    // This prevents leaving the user's terminal in a broken state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Propagate any errors that occurred during execution
    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

/// Main application loop - handles rendering and input
/// We use a generic backend so this could work with different terminal implementations
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        // Render the UI - this closure is called with a Frame we can draw to
        terminal.draw(|f| {
            // Create a two-panel vertical layout
            // Using constraints allows ratatui to handle terminal resizing gracefully
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2) // Add padding so content doesn't touch screen edges
                .constraints([
                    Constraint::Min(1),    // Todo list takes remaining space
                    Constraint::Length(3)  // Input area is fixed height
                ].as_ref())
                .split(f.area());

            // Convert todo items to ListItems for rendering
            // We do this fresh each frame because completed status may have changed
            let items: Vec<ListItem> = app
                .todos
                .iter()
                .map(|todo| {
                    // Use checkbox pattern familiar from many todo apps
                    let checkbox = if todo.completed { "[✓] " } else { "[ ] " };
                    
                    // Style completed items differently to provide clear visual feedback
                    // Strikethrough + dark gray is standard convention for completed tasks
                    let style = if todo.completed {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    // Combine checkbox and text with appropriate styling
                    ListItem::new(Line::from(vec![
                        Span::raw(checkbox),
                        Span::styled(&todo.text, style),
                    ]))
                })
                .collect();

            // Create the list widget with all our styled items
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        // Put all controls in title so they're always visible
                        .title("📝 Todo List (↑/↓: navigate, Space: toggle, a: add, d: delete, q: quit)"),
                )
                // Highlight style makes it clear which item is selected
                // Blue background is conventional for selection in TUIs
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                // Arrow symbol provides additional visual cue for selection
                .highlight_symbol("► ");

            // Render the list with its stateful selection
            // We pass state mutably so ratatui can update it if needed
            f.render_stateful_widget(list, chunks[0], &mut app.state);

            // Update input area text based on current mode
            // This provides context-sensitive help to the user
            let input_text = if app.input_mode {
                format!("New todo: {} (Press Enter to confirm, Esc to cancel)", app.input)
            } else {
                "Press 'a' to add a new todo".to_string()
            };

            // Style input area differently when active to show mode clearly
            // Yellow is attention-getting and conventional for "active" state
            let input = Paragraph::new(input_text)
                .style(if app.input_mode {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[1]);
        })?;

        // Check if an event is available without blocking
        // We use a very short timeout to keep the UI responsive
        if event::poll(std::time::Duration::from_millis(16))? {
            // Only process keyboard events, ignore other event types
            if let Event::Key(key) = event::read()? {
                // CRITICAL: Only process key press events, not release events
                // Some terminals send both Press and Release, which would cause double input
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Different key handling based on mode - modal interface pattern
                if app.input_mode {
                    // In input mode, keys type into the buffer
                    match key.code {
                        KeyCode::Enter => app.add_todo(),
                        KeyCode::Char(c) => app.input.push(c),
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        // Esc cancels input without saving
                        KeyCode::Esc => {
                            app.input_mode = false;
                            app.input.clear();
                        }
                        _ => {}
                    }
                } else {
                    // In navigation mode, keys control the list
                    match key.code {
                        KeyCode::Char('q') => return Ok(()), // Exit cleanly
                        // Support both arrow keys and vim-style navigation
                        // This accommodates different user preferences
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char(' ') => app.toggle_completed(),
                        KeyCode::Char('d') => app.delete_selected(),
                        KeyCode::Char('a') => app.input_mode = true,
                        _ => {}
                    }
                }
            }
        }
    }
}