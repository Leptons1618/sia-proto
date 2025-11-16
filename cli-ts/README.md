# SIA CLI - TypeScript TUI

TypeScript-based Terminal User Interface for the System Insight Agent, built with [Ink](https://github.com/vadimdemedes/ink) (React for CLIs).

## Features

- 🎨 Beautiful interactive TUI with React components
- ⌨️ Command autocomplete and suggestions
- 📊 Formatted status, list, and event detail views
- 🔌 Unix socket IPC communication
- 🎯 Both interactive and command-line modes

## Installation

```bash
cd cli-ts
npm install
npm run build
```

## Usage

### Interactive Mode

```bash
npm run dev
# or
node dist/index.js
```

### Command Mode

```bash
node dist/index.js status
node dist/index.js list 20
node dist/index.js show <event-id>
node dist/index.js analyze <event-id>
```

## Development

```bash
# Install dependencies
npm install

# Run in development mode (with hot reload)
npm run dev

# Build for production
npm run build

# Run built version
npm start
```

## Technology Stack

- **Ink** - React for CLIs
- **TypeScript** - Type-safe development
- **Chalk** - Terminal colors and styling
- **Node.js** - Runtime environment

## References

- [Ink Documentation](https://github.com/vadimdemedes/ink)
- [GitHub Copilot CLI](https://docs.github.com/en/copilot/concepts/agents/about-copilot-cli)
- [Gemini CLI](https://geminicli.com/docs/)

