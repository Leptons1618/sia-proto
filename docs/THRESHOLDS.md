# Configuring CPU and Memory Thresholds

The SIA agent uses configurable thresholds to trigger events when CPU or memory usage exceeds certain levels.

## Configuration File

Thresholds are configured in `config/default.toml` under the `[agent.thresholds]` section:

```toml
[agent.thresholds]
# CPU usage thresholds (percentage)
cpu_warning = 80.0      # Warning threshold - triggers WARNING event
cpu_critical = 95.0     # Critical threshold - triggers CRITICAL event
cpu_sustained_count = 2 # Number of consecutive checks before warning

# Memory usage thresholds (percentage)
memory_warning = 85.0   # Warning threshold - triggers WARNING event
memory_critical = 95.0  # Critical threshold - triggers CRITICAL event
```

## How It Works

### CPU Thresholds
- **Warning**: When CPU usage exceeds `cpu_warning` for `cpu_sustained_count` consecutive checks, a WARNING event is generated
- **Critical**: When CPU usage exceeds `cpu_critical`, a CRITICAL event is immediately generated

### Memory Thresholds
- **Warning**: When memory usage exceeds `memory_warning`, a WARNING event is generated
- **Critical**: When memory usage exceeds `memory_critical`, a CRITICAL event is generated

## Changing Thresholds

1. **Edit the config file**: Open `config/default.toml` and modify the threshold values
2. **Restart the agent**: The agent must be restarted for changes to take effect
3. **Verify**: Use `/status` or `/config` in the CLI to see the current thresholds

## Example Configuration

For a more sensitive system (triggers earlier):

```toml
[agent.thresholds]
cpu_warning = 70.0
cpu_critical = 90.0
cpu_sustained_count = 2
memory_warning = 75.0
memory_critical = 90.0
```

For a less sensitive system (triggers later):

```toml
[agent.thresholds]
cpu_warning = 85.0
cpu_critical = 98.0
cpu_sustained_count = 3
memory_warning = 90.0
memory_critical = 98.0
```

## Viewing Current Thresholds

### In the CLI:
- Use `/status` to see thresholds in the status output
- Use `/config` to see a detailed view of all thresholds

### Default Values:
If thresholds are not specified in the config file, the following defaults are used:
- CPU Warning: 80%
- CPU Critical: 95%
- CPU Sustained Count: 2
- Memory Warning: 85%
- Memory Critical: 95%

## Troubleshooting

If thresholds show as "not available":
1. **Check config file**: Ensure `config/default.toml` exists and has the `[agent.thresholds]` section
2. **Restart agent**: The agent must be restarted after changing the config
3. **Check logs**: Look for config loading errors in the agent logs
4. **Verify path**: The agent looks for config at `./config/default.toml` by default, or the path specified by `SIA_CONFIG` environment variable

