# Agent Integration Guide

This guide explains how to integrate the Slack MCP Server with AI assistants and other MCP clients.

## What is MCP?

The Model Context Protocol (MCP) is an open protocol that standardizes how applications provide context to Large Language Models (LLMs). It enables AI assistants to interact with external data sources and tools in a consistent, secure way.

## Quick Start

### Configuration for Claude Desktop

Add this server to your Claude Desktop configuration file:

**macOS/Linux**: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "slack": {
      "command": "/path/to/slack_mcp/target/release/slack_mcp",
      "env": {
        "SLACK_TOKEN": "xoxb-your-slack-bot-token"
      }
    }
  }
}
```

Or, if using the config file approach:

```json
{
  "mcpServers": {
    "slack": {
      "command": "/path/to/slack_mcp/target/release/slack_mcp"
    }
  }
}
```

### Configuration for Other MCP Clients

The server implements the standard MCP protocol and should work with any compliant client. Consult your client's documentation for how to add MCP servers.

## Available Tools

### `read_thread`

Retrieves a complete Slack thread conversation including the original message and all replies.

#### Parameters

| Parameter | Type   | Required | Description                           |
|-----------|--------|----------|---------------------------------------|
| `url`     | string | Yes      | Full Slack thread URL                 |

#### Example Usage

```json
{
  "tool": "read_thread",
  "parameters": {
    "url": "https://alvys.slack.com/archives/C0982014L82/p1762872519417859"
  }
}
```

#### Response Format

The tool returns the thread conversation as formatted text, including:
- Original message author and timestamp
- Message text content
- All thread replies with authors and timestamps
- Threaded conversation structure

#### Example Response

```
[2024-01-15 10:30:15] @john.doe
Hey team, quick question about the deployment schedule.

    [2024-01-15 10:32:45] @jane.smith
    We're planning to deploy on Friday afternoon.
    
    [2024-01-15 10:35:20] @john.doe
    Sounds good, I'll prepare the release notes.
```

## Common Use Cases

### 1. Analyzing Team Discussions

Ask your AI assistant to analyze a Slack thread for action items:

```
Please read this Slack thread and summarize the key decisions and action items:
https://alvys.slack.com/archives/C0982014L82/p1762872519417859
```

### 2. Extracting Information

Get specific information from a conversation:

```
From this thread https://alvys.slack.com/archives/C0982014L82/p1762872519417859,
what was the final decision on the database migration approach?
```

### 3. Thread Summarization

Generate concise summaries of long discussions:

```
Please provide a brief summary of this discussion:
https://alvys.slack.com/archives/C0982014L82/p1762872519417859
```

### 4. Context for Follow-ups

Use thread content as context for drafting responses:

```
Based on this thread https://alvys.slack.com/archives/C0982014L82/p1762872519417859,
draft a follow-up message addressing the concerns raised.
```

## Understanding Slack URLs

Slack thread URLs follow this pattern:

```
https://{workspace}.slack.com/archives/{channel_id}/p{thread_ts}
```

### Components

- **Workspace**: Your Slack workspace subdomain (e.g., `alvys`)
- **Channel ID**: Unique identifier for the channel (e.g., `C0982014L82`)
  - Starts with `C` for public channels
  - Starts with `G` for private channels
  - Starts with `D` for direct messages
- **Thread Timestamp**: Message timestamp with the decimal point removed (e.g., `p1762872519417859`)
  - The `p` prefix indicates a permalink
  - The number is the Unix timestamp in microseconds

### Finding Thread URLs

1. **From Slack Desktop/Web**:
   - Hover over any message
   - Click the three-dot menu (⋯)
   - Select "Copy link"

2. **From Slack Mobile**:
   - Long-press on a message
   - Select "Share" → "Copy link"

## Permissions and Security

### Required Slack Scopes

Your Slack bot needs these OAuth scopes:

- `channels:history` - Read public channel messages
- `groups:history` - Read private channel messages (if needed)
- `im:history` - Read direct messages (if needed)
- `mpim:history` - Read group direct messages (if needed)

### Privacy Considerations

- The bot can only access channels it has been added to
- Users should be aware that thread content will be shared with AI assistants
- Consider your organization's data privacy policies
- Sensitive information should be handled according to your security guidelines

### Best Practices

1. **Limit Bot Access**: Only add the bot to channels where it's needed
2. **Token Security**: Store tokens securely, never in code or public repositories
3. **Audit Usage**: Monitor which threads are being accessed
4. **User Consent**: Ensure team members are aware of AI assistant access
5. **Data Retention**: Consider how long thread data is retained by AI services

## Troubleshooting

### "Token not found" Error

**Solution**: Ensure `SLACK_TOKEN` is set in either:
- Environment variable
- `~/.slackmcp.config` file

### "Channel not found" Error

**Possible causes**:
- Bot hasn't been added to the channel
- Invalid channel ID in URL
- Channel was deleted or archived

**Solution**: Add the bot to the channel using `/invite @your-bot-name`

### "Missing permissions" Error

**Solution**: Review and add required OAuth scopes in your Slack App settings

### Invalid URL Format

**Solution**: Ensure the URL follows the correct Slack permalink format. Use Slack's "Copy link" feature to get the correct URL.

## Advanced Usage

### Scripting with MCP

If you're building custom tools or scripts that use MCP:

```python
import mcp

# Connect to the Slack MCP server
client = mcp.Client("slack")

# Read a thread
result = await client.call_tool(
    "read_thread",
    url="https://alvys.slack.com/archives/C0982014L82/p1762872519417859"
)

print(result)
```

### Batch Processing

Process multiple threads sequentially:

```
Please analyze these three threads and identify common themes:
1. https://alvys.slack.com/archives/C0982014L82/p1762872519417859
2. https://alvys.slack.com/archives/C0982014L82/p1762872589123456
3. https://alvys.slack.com/archives/C0982014L82/p1762872612345678
```

## Future Capabilities

The following features are planned for future releases:

- **Attachment Retrieval**: Download and analyze images, documents, and other files
- **User Information**: Resolve user IDs to full names and profiles
- **Reaction Data**: Access emoji reactions and their authors
- **Search**: Find threads by keywords, date ranges, or participants
- **Write Operations**: Post messages and replies (separate write-enabled version)

## Support and Feedback

If you encounter issues or have suggestions for improving agent integration:

1. Check the [README.md](README.md) for basic troubleshooting
2. Review Slack API documentation
3. Open an issue on the GitHub repository
4. Provide example URLs and error messages when reporting problems

## Resources

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [Slack API Documentation](https://api.slack.com/)
- [Claude Desktop MCP Guide](https://docs.anthropic.com/claude/docs/mcp)

---

**Note**: This server is read-only and cannot post messages, modify content, or perform any write operations on your Slack workspace.

