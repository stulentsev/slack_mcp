# Slack MCP Server

A lightweight Model Context Protocol (MCP) server that provides read-only access to Slack conversations. This server enables Cursor's AI assistant to retrieve and analyze Slack thread conversations directly from your workspace.

## What It Does

The Slack MCP Server exposes a single tool called `read_thread` that allows Cursor to:

- **Read Slack Threads**: Fetch complete thread conversations including the original message and all replies
- **Parse Slack URLs**: Automatically extracts channel and thread IDs from Slack permalink URLs
- **Format Conversations**: Returns thread content in a readable format with timestamps and authors

### Available Tool: `read_thread`

**Parameters:**
- `url` (string, required): The full Slack thread URL

**Returns:**
- Complete thread conversation text including all replies, formatted with timestamps and author information

**Example Slack URL format:**
```
https://alvys.slack.com/archives/C0982014L82/p1762872519417859
```

## Installation

### Step 1: Install Rust Toolchain

If you don't have Rust installed, follow the instructions for your operating system:

#### macOS or Linux

Run this command in your terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts (press Enter to accept defaults). After installation, restart your terminal or run:

```bash
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version
cargo --version
```

#### Windows

1. Download and run the Rust installer from: https://rustup.rs/
2. Follow the installation wizard (accept defaults)
3. Restart your terminal/PowerShell
4. Verify installation:

```powershell
rustc --version
cargo --version
```

**Note**: On Windows, you may need to install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) if prompted during Rust installation.

### Step 2: Build the Server

Clone the repository and build the server:

```bash
# Clone the repository
git clone https://github.com/stulentsev/slack_mcp
cd slack_mcp

# Install the binary to Cargo's bin directory
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/slack_mcp` (macOS/Linux) or `%USERPROFILE%\.cargo\bin\slack_mcp.exe` (Windows). This directory is automatically added to your PATH when you install Rust.

### Step 3: Get Your Slack Token

This integration uses a shared company Slack bot token (not a personal token). To get it:

1. Open your company's **Keeper Vault**
2. Search for "Slack - MCP Server (Readonly)"
3. Copy the token (it should start with `xoxb-`)

This is a shared, read-only bot token provisioned specifically for this MCP integration. Do not create your own Slack app — use this one.

### Step 4: Create Configuration File

Create the configuration file:

**macOS/Linux:**
```bash
# Create the config directory
mkdir -p ~/.config/slackmcp

# Create the config file
cat > ~/.config/slackmcp/config << EOF
SLACK_TOKEN=xoxb-your-token-from-keeper-vault
EOF

# Secure the file (restrict access to your user only)
chmod 600 ~/.config/slackmcp/config
```

**Windows:**
```powershell
# Create the config directory
New-Item -ItemType Directory -Force -Path $env:USERPROFILE\.config\slackmcp

# Create the config file
$token = "xoxb-your-token-from-keeper-vault"
"SLACK_TOKEN=$token" | Out-File -FilePath $env:USERPROFILE\.config\slackmcp\config -Encoding utf8 -NoNewline
```

**Important**: Replace `xoxb-your-token-from-keeper-vault` with the actual token you copied from Keeper.

The config path follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/). You can override it by setting `$XDG_CONFIG_HOME`. The legacy path `~/.slackmcp.config` is still supported as a fallback.

### Step 5: Configure Cursor

1. Open Cursor
2. Open the MCP settings (usually found in Settings → Features → Model Context Protocol)
3. Add the following configuration to your MCP servers list:

```json
{
  "mcpServers": {
    "slack": {
      "command": "/Users/YOUR_USERNAME/.cargo/bin/slack_mcp"
    }
  }
}
```

**Note**: 
- **macOS/Linux**: Replace `YOUR_USERNAME` with your actual username. You can find it by running `whoami` in Terminal. The path will be `~/.cargo/bin/slack_mcp`.
- **Windows**: Use the Windows path format, e.g., `C:\Users\YOUR_USERNAME\.cargo\bin\slack_mcp.exe`

Alternatively, if `~/.cargo/bin` (or `%USERPROFILE%\.cargo\bin` on Windows) is in your PATH, you can use just `slack_mcp` as the command.

### Step 6: Restart Cursor

Restart Cursor completely for the MCP server configuration to take effect.

## Usage

Once installed and configured, you can use the Slack MCP server in Cursor by asking it to read Slack threads. For example:

```
Can you read this Slack thread and summarize the key points?
https://alvys.slack.com/archives/C0982014L82/p1762872519417859
```

Or:

```
From this thread https://alvys.slack.com/archives/C0982014L82/p1762872519417859,
what was the final decision on the deployment schedule?
```

## Troubleshooting

### "Token not found" Error

**Solution**: Ensure your config file exists and contains the correct token. Verify the file path and permissions:

```bash
ls -la ~/.config/slackmcp/config
cat ~/.config/slackmcp/config
```

### "Channel not found" Error

**Possible causes**:
- The Slack bot hasn't been added to the channel
- Invalid channel ID in the URL

**Solution**: Ask your team lead to add the Slack bot to the channel using `/invite @slack-bot-name`

### Binary Not Found

**Solution**: Verify the binary path in your Cursor MCP configuration matches where you placed the file:

**macOS/Linux:**
```bash
# Check if the binary exists and is executable
ls -la ~/.cargo/bin/slack_mcp

# Test running it directly
~/.cargo/bin/slack_mcp
```

**Windows:**
```powershell
# Check if the binary exists
Test-Path $env:USERPROFILE\.cargo\bin\slack_mcp.exe

# Test running it directly
& $env:USERPROFILE\.cargo\bin\slack_mcp.exe
```

If the binary doesn't exist at that path, update your Cursor MCP configuration with the correct path.

### Build Errors

**Solution**: If you encounter build errors:

1. Ensure you have the latest Rust toolchain:
   ```bash
   rustup update
   ```

2. On Linux, you may need additional system dependencies:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libssl-dev
   
   # Fedora/RHEL
   sudo dnf install gcc pkg-config openssl-devel
   ```

3. On macOS, ensure you have Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

### MCP Server Not Appearing in Cursor

**Solution**: 
1. Verify your Cursor MCP configuration JSON is valid
2. Ensure you've restarted Cursor completely (quit and reopen)
3. Check Cursor's MCP server logs for error messages

## Security

**Important**:
- Your `~/.config/slackmcp/config` file contains a shared company credential. Keep it secure and never share it outside the organization.
- The file permissions (`chmod 600`) ensure only you can read it.
- Never commit this file to version control.

## How It Works

The server implements the Model Context Protocol (MCP) standard, communicating with Cursor via JSON-RPC over stdin/stdout. When Cursor needs to read a Slack thread, it sends a request to the server, which:

1. Parses the Slack URL to extract channel ID and thread timestamp
2. Authenticates with Slack API using your bot token
3. Fetches the thread conversation
4. Formats and returns the conversation text to Cursor

The server is read-only and cannot post messages, modify content, or perform any write operations on your Slack workspace.
