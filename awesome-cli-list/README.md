# Awesome List CLI Browser

A command-line interface (CLI) application to browse and interact with the [Awesome List](https://github.com/sindresorhus/awesome) directly from your terminal.

## Features

- Fetch and parse the Awesome List README from GitHub
- Display a navigable list of topics and sub-topics
- Search functionality to filter topics
- Open links directly in your default web browser
- User-friendly terminal UI with keyboard navigation

## Installation

1. Ensure you have Rust installed on your system. If not, you can install it from [https://www.rust-lang.org/](https://www.rust-lang.org/).

2. Clone this repository:

3. Build the project:

4. The executable will be available in the `target/release` directory.

## Usage

Run the application:

### Navigation

- Use `Up` and `Down` arrow keys to navigate through the list
- Press `Enter` to open the selected link in your default web browser
- Use `Tab` to toggle visibility of sub-links
- Type to search and filter the list
- Press `Esc` to show the quit dialog

## Dependencies

- `comrak`: For parsing Markdown content
- `crossterm`: For terminal manipulation and event handling
- `ratatui`: For creating the terminal user interface
- `reqwest`: For making HTTP requests
- `tokio`: For asynchronous runtime
- `webbrowser`: For opening URLs in the default web browser

## TODO

1. Add sub-links right in the terminal so users can land on appropriate topics

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).
