# beetle VS Code Extension

The beetle VS Code extension brings lightning-fast source code search capabilities directly to your editor. Built on top of the powerful beetle search engine, this extension allows you to create indexes of your codebases and perform blazing-fast searches without leaving VS Code.

## Features

- **🚀 Lightning Fast Search**: Powered by Tantivy full-text search engine
- **📁 Index Management**: Create, list, and manage multiple code indexes
- **🔍 Integrated Search Panel**: Dedicated search interface in the activity bar
- **📋 Search Results Navigation**: Click to jump directly to search results
- **⚙️ Configurable Settings**: Customize executable path, index storage, and more
- **🔄 Auto-Index Workspaces**: Optionally auto-create indexes for new workspaces

## Installation

### Prerequisites

You need to have the beetle CLI tool installed on your system. You can install it from source:

```bash
git clone https://github.com/yourusername/beetle.git
cd beetle
cargo install --path apps/cli
```

Make sure `beetle` is available in your PATH, or configure the executable path in the extension settings.

### Install Extension

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "beetle Code Search"
4. Click Install

## Quick Start

### 1. Create an Index

Before you can search, you need to create an index of your codebase:

1. Open the Command Palette (Ctrl+Shift+P)
2. Run "beetle: Create Index"
3. Enter a name for your index
4. Enter the path to your codebase (defaults to current workspace)

### 2. Search Your Code

Once you have an index:

1. Click the beetle icon in the activity bar, or
2. Open Command Palette and run "beetle: Search Code"
3. Select an index to search
4. Enter your search query
5. Browse results in the search panel

## Commands

- **beetle: Search Code** - Search across your indexed codebases
- **beetle: Create Index** - Create a new search index
- **beetle: List Indexes** - View all available indexes
- **beetle: Delete Index** - Remove an index (when supported)
- **beetle: Open Search Panel** - Open the beetle activity panel

## Configuration

Configure beetle through VS Code settings:

```json
{
  "beetle.executablePath": "beetle",
  "beetle.defaultIndexPath": "/path/to/your/indexes",
  "beetle.maxResults": 50,
  "beetle.autoCreateIndex": false
}
```

### Settings

- **beetle.executablePath**: Path to the beetle executable (default: "beetle")
- **beetle.defaultIndexPath**: Default directory for storing indexes
- **beetle.maxResults**: Maximum number of search results to display
- **beetle.autoCreateIndex**: Automatically create indexes for new workspaces

## Usage Tips

### Search Queries

beetle supports various search patterns:

- **Function names**: `function parseJSON`
- **Multiple terms**: `Result Err unwrap`
- **TODO comments**: `TODO FIXME`
- **Error patterns**: `panic! unwrap() expect()`

### Best Practices

1. **Create separate indexes** for different projects
2. **Use descriptive names** for your indexes
3. **Keep indexes updated** by recreating them when your codebase changes significantly
4. **Use specific search terms** for better results

## Keyboard Shortcuts

No default keyboard shortcuts are provided, but you can set your own:

1. Go to File > Preferences > Keyboard Shortcuts
2. Search for "beetle"
3. Assign shortcuts to your favorite commands

## Troubleshooting

### "Command not found: beetle"

Make sure the beetle CLI is installed and available in your PATH, or set the correct path in `beetle.executablePath`.

### "No indexes found"

Create an index first using "beetle: Create Index" command.

### Search returns no results

- Verify your search terms
- Make sure the index contains the files you're looking for
- Try broader search terms

## Contributing

This extension is part of the beetle project. Contributions are welcome!

## License

MIT License - see the main beetle project for details.
