# Slack MCP Server

A lightweight Model Context Protocol (MCP) server that provides read-only access to Slack conversations. This server enables AI assistants and other MCP clients to retrieve and analyze Slack thread conversations.

## Features

- **Thread Reading**: Fetch complete thread conversations from Slack
- **URL Parsing**: Automatically extracts channel and thread IDs from Slack URLs
- **Text-Only Mode**: Currently retrieves text content (attachment support planned)
- **Flexible Configuration**: Supports environment variables or config file

## Installation

### Prerequisites

- Rust 1.70+ (2024 edition)
- A Slack workspace with API access
- Slack Bot Token with appropriate permissions

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/slack_mcp.git
cd slack_mcp

# Build the project
cargo build --release

# The binary will be available at target/release/slack_mcp
```

### Running

```bash
# Run directly with cargo
cargo run

# Or run the compiled binary
./target/release/slack_mcp
```

## Configuration

The server requires a Slack Bot Token to authenticate with the Slack API. You can provide this token in two ways:

### Option 1: Environment Variable

```bash
export SLACK_TOKEN="xoxb-your-slack-bot-token"
./slack_mcp
```

### Option 2: Configuration File

Create a configuration file at `~/.slackmcp.config` with the following content:

```
SLACK_TOKEN=xoxb-your-slack-bot-token
```

The server will automatically load this file using dotenv.

## Obtaining a Slack Token

1. Go to [Slack API: Your Apps](https://api.slack.com/apps)
2. Create a new app or select an existing one
3. Navigate to "OAuth & Permissions"
4. Add the following Bot Token Scopes:
   - `channels:history` - View messages in public channels
   - `groups:history` - View messages in private channels
   - `im:history` - View messages in direct messages
   - `mpim:history` - View messages in group direct messages
5. Install the app to your workspace
6. Copy the "Bot User OAuth Token" (starts with `xoxb-`)

## Usage

### Slack URL Format

The server accepts Slack thread URLs in the following format:

```
https://{workspace}.slack.com/archives/{channel_id}/p{thread_ts}
```

Example:
```
https://alvys.slack.com/archives/C0982014L82/p1762872519417859
```

Where:
- `C0982014L82` is the channel ID
- `p1762872519417859` is the thread timestamp (with the period removed)

### MCP Tool: `read_thread`

The server exposes a single tool called `read_thread` that retrieves thread conversations.

**Parameters:**
- `url` (string, required): The Slack thread URL

**Returns:**
- Thread conversation text including all replies

See [AGENTS.md](AGENTS.md) for detailed usage examples with AI assistants.

## Development

### Project Structure

```
slack_mcp/
├── Cargo.toml          # Rust dependencies and project metadata
├── src/
│   └── main.rs         # Main server implementation
├── README.md           # This file
└── AGENTS.md           # MCP client/agent integration guide
```

### Adding Dependencies

Edit `Cargo.toml` to add required dependencies:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
dotenv = "0.15"
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Roadmap

- [x] Basic thread reading functionality
- [ ] Attachment download support
- [ ] User information resolution
- [ ] Reaction data retrieval
- [ ] Search functionality
- [ ] Rate limiting and caching
- [ ] Multi-workspace support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT License](LICENSE)

## Support

For issues, questions, or contributions, please open an issue on the GitHub repository.

## Security

**Important**: Keep your Slack tokens secure. Never commit tokens to version control or share them publicly. The `.gitignore` file is configured to exclude `.env` and configuration files.

## Acknowledgments

Built using the [Model Context Protocol](https://modelcontextprotocol.io/) specification.

