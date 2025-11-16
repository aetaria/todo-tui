# 📝 Rust TUI Todo List

A lightweight, terminal-based todo list application built with Rust. Manage your tasks directly from the command line with an intuitive keyboard-driven interface.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-%23121011.svg?style=for-the-badge&logo=gnu-bash&logoColor=white)

## ✨ Features

- 🎯 **Simple & Fast**: Lightweight terminal interface with instant startup
- ⌨️ **Keyboard-driven**: Full navigation and control without touching the mouse
- ✅ **Task Management**: Add, complete, and delete todos with ease
- 💾 **Persistent Storage**: Todos are automatically saved to disk between sessions
- 🎨 **Visual Feedback**: Clear indicators for completed tasks with strikethrough styling
- 🚀 **Zero Config**: Works out of the box, no configuration needed

## 📦 Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rust-tui-todo.git
cd rust-tui-todo
```

2. Add dependencies to `Cargo.toml`:
```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

3. Build and run:
```bash
cargo run
```

Or build a release version:
```bash
cargo build --release
./target/release/rust-tui-todo
```

## 🎮 Usage

### Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Space` | Toggle todo completion |
| `a` | Add new todo |
| `d` | Delete selected todo |
| `q` | Quit application |

### Adding a Todo

1. Press `a` to enter input mode
2. Type your todo text
3. Press `Enter` to confirm or `Esc` to cancel

### Completing Todos

Navigate to a todo with arrow keys and press `Space` to mark it as complete. Completed todos are shown with a checkmark `[✓]` and strikethrough text.

### Data Persistence

All todos are automatically saved to `todos.json` in the directory where you run the application. Changes are saved immediately after:
- Adding a new todo
- Toggling completion status
- Deleting a todo

This means your todos will persist between sessions, and you can have different todo lists for different projects by running the app from different directories.

## 🏗️ Project Structure

```
.
├── src/
│   └── main.rs          # Main application code
├── Cargo.toml           # Project dependencies
└── README.md            # This file
```

## 🛠️ Technology Stack

- **[Ratatui](https://github.com/ratatui-org/ratatui)**: Terminal UI framework for creating rich text interfaces
- **[Crossterm](https://github.com/crossterm-rs/crossterm)**: Cross-platform terminal manipulation library
- **[Serde](https://serde.rs/)**: Serialization framework for converting Rust data structures to/from various formats
- **[Serde JSON](https://github.com/serde-rs/json)**: JSON serialization/deserialization support for persistent storage

## 🚀 Future Enhancements

Potential features for future versions:

- [x] Persistent storage (save todos to file)
- [ ] Categories and tags
- [ ] Due dates and reminders
- [ ] Priority levels
- [ ] Search and filter functionality
- [ ] Multiple todo lists
- [ ] Export to various formats
- [ ] Undo/redo functionality
- [ ] Cloud sync support

## 🤝 Contributing

Contributions are welcome! Feel free to:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui)
- Inspired by terminal productivity tools
- Thanks to the Rust community for excellent tooling and libraries

## 📧 Contact

Evan Kae

Project Link: [https://github.com/aetaria/todo-tui](https://github.com/aetaria/todo-tui)

---

Made with ❤️ and Rust