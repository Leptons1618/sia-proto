import chalk from 'chalk';
import type { IpcResponse } from './ipc.js';

export function showGreeting(): string {
  return `
  ${chalk.cyan('███████╗██╗ █████╗ ')}
  ${chalk.cyan('██╔════╝██║██╔══██╗')}
  ${chalk.cyan('███████╗██║███████║')}
  ${chalk.cyan('╚════██║██║██╔══██║')}
  ${chalk.cyan('███████║██║██║  ██║')}
  ${chalk.cyan('╚══════╝╚═╝╚═╝  ╚═╝')}
  ${chalk.cyan.dim('     CLI - System Insight Agent')}

${chalk.white.dim('  Type /help for available commands')}

`;
}

// Helper function to format bytes to human-readable format
function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  
  return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

// Helper function to format percentage with visual indicator (fixed width)
function formatPercentage(value: number, color: string): string {
  const barLength = 15;
  const filled = Math.round((value / 100) * barLength);
  const empty = barLength - filled;
  const bar = '█'.repeat(filled) + '░'.repeat(empty);
  return `${value.toFixed(1).padStart(5)}% ${chalk.dim('[' + bar + ']')}`;
}

export function printStatus(response: IpcResponse, width: number = 63): string {
  if (!response.success) {
    const error = response.data?.error || 'Unknown error';
    return `${chalk.red.bold('❌ Error:')} ${error}`;
  }

  const data = response.data;
  const uptime = data.uptime_seconds || 0;
  const uptimeStr = formatUptime(uptime);

  // Backend returns raw metrics - we format them here
  const cpuUsage = (data.metrics?.cpu_usage || 0) as number;
  const memUsedMB = data.metrics?.memory_used_mb || 0;
  const memTotalMB = data.metrics?.memory_total_mb || 0;
  const memPercent = (data.metrics?.memory_percent || 0) as number;

  // Determine colors based on thresholds
  const cpuColor = cpuUsage > 80 ? 'red' : cpuUsage > 60 ? 'yellow' : 'green';
  const memColor = memPercent > 85 ? 'red' : memPercent > 70 ? 'yellow' : 'green';

  const status = data.status || 'unknown';
  const cpuStatus = data.collectors?.cpu || 'unknown';
  const memStatus = data.collectors?.memory || 'unknown';
  const critical = data.events?.critical || 0;
  const warning = data.events?.warning || 0;
  const info = data.events?.info || 0;

  // Use full width with small margin
  const boxWidth = Math.max(width - 2, 59);
  const contentWidth = boxWidth - 2; // -2 for borders (│ │)
  
  // Format metrics for display (backend provides raw data, we format here)
  const cpuUsageFormatted = formatPercentage(cpuUsage, cpuColor);
  const memUsedFormatted = formatBytes(memUsedMB * 1024 * 1024);
  const memTotalFormatted = formatBytes(memTotalMB * 1024 * 1024);
  const memPercentFormatted = formatPercentage(memPercent, memColor);
  
  // Fixed width for memory details (left side) and percentage (right side)
  const memDetailsWidth = 25; // Fixed width for "used / total"
  const memDetails = `${memUsedFormatted} / ${memTotalFormatted}`.padEnd(memDetailsWidth);

  const title = 'SIA Agent Status';
  const titlePadding = Math.floor((boxWidth - title.length) / 2);
  
  // Helper function to create a line with proper padding
  // Strips ANSI codes for length calculation
  const stripAnsi = (str: string) => str.replace(/\u001b\[[0-9;]*m/g, '');
  const makeLine = (leftText: string, rightText: string = '') => {
    const leftLen = stripAnsi(leftText).length;
    const rightLen = stripAnsi(rightText).length;
    const padding = Math.max(0, contentWidth - leftLen - rightLen);
    return `${chalk.cyan('║')} ${leftText}${' '.repeat(padding)}${rightText} ${chalk.cyan('║')}`;
  };
  
  return `
${chalk.cyan('╔' + '═'.repeat(boxWidth) + '╗')}
${chalk.cyan.bold('║' + ' '.repeat(titlePadding) + title + ' '.repeat(boxWidth - title.length - titlePadding) + '║')}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('Uptime:', chalk.white(uptimeStr))}
${makeLine('Status:', chalk.green.bold(status))}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('System Metrics:', '')}
${makeLine('  CPU Usage:', chalk[cpuColor].bold(cpuUsageFormatted))}
${makeLine('  Memory:', chalk[memColor](memDetails) + ' ' + chalk[memColor].bold(memPercentFormatted))}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('Collectors:', '')}
${makeLine('  CPU:', chalk.green(`✓ ${cpuStatus}`))}
${makeLine('  Memory:', chalk.green(`✓ ${memStatus}`))}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('Events (open):', '')}
${makeLine('  Critical:', chalk.red.bold(String(critical)))}
${makeLine('  Warning:', chalk.yellow.bold(String(warning)))}
${makeLine('  Info:', chalk.blue.bold(String(info)))}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('Thresholds:', '')}
${data.thresholds ? 
  makeLine('  CPU Warning:', chalk.yellow(`${data.thresholds.cpu_warning}%`)) + '\n' +
  makeLine('  CPU Critical:', chalk.red(`${data.thresholds.cpu_critical}%`)) + '\n' +
  makeLine('  Memory Warning:', chalk.yellow(`${data.thresholds.memory_warning}%`)) + '\n' +
  makeLine('  Memory Critical:', chalk.red(`${data.thresholds.memory_critical}%`))
  : makeLine('  (not available)', '')}
${chalk.cyan('╚' + '═'.repeat(boxWidth) + '╝')}

`;
}

export function printList(response: IpcResponse, width: number = 80): string {
  if (!response.success) {
    const error = response.data?.error || 'Unknown error';
    return `${chalk.red.bold('❌ Error:')} ${error}`;
  }

  const events = response.data?.events || [];
  
  if (events.length === 0) {
    return `\n${chalk.dim('📭 No events found.')}\n`;
  }

  let output = `\n${chalk.cyan('┌──────────────────────────┬────────────────────────┬──────────┬───────────────┬────────┐')}\n`;
  output += `${chalk.cyan.bold('│ Event ID                 │ Timestamp              │ Severity │ Type          │ Status │')}\n`;
  output += `${chalk.cyan('├──────────────────────────┼────────────────────────┼──────────┼───────────────┼────────┤')}\n`;

  for (const event of events) {
    const eventId = truncate(event.event_id || '?', 24);
    const ts = event.ts || 0;
    const tsStr = formatTimestamp(ts);
    const severity = event.severity || '?';
    const type = truncate(event.type || '?', 13);
    const status = event.status || '?';

    const severityColor = severity === 'CRITICAL' ? 'red' : severity === 'WARNING' ? 'yellow' : 'blue';
    
    output += `│ ${chalk.white(eventId.padEnd(24))} │ ${chalk.dim(tsStr.padEnd(22))} │ ${chalk[severityColor].bold(severity.padEnd(8))} │ ${chalk.white(type.padEnd(13))} │ ${chalk.green(status.padEnd(6))} │\n`;
  }

  output += `${chalk.cyan('└──────────────────────────┴────────────────────────┴──────────┴───────────────┴────────┘')}\n`;

  return output;
}

export function printShow(response: IpcResponse, width: number = 63): string {
  if (!response.success) {
    const error = response.data?.error || 'Unknown error';
    return `${chalk.red.bold('❌ Error:')} ${error}`;
  }

  const event = response.data;
  const ts = event.ts || 0;
  const tsStr = formatTimestamp(ts);
  const severity = event.severity || '?';
  const severityColor = severity === 'CRITICAL' ? 'red' : severity === 'WARNING' ? 'yellow' : 'blue';

  let output = `\n${chalk.cyan('╔═══════════════════════════════════════════════════════════════╗')}\n`;
  output += `${chalk.cyan.bold('║                      Event Details                            ║')}\n`;
  output += `${chalk.cyan('╠═══════════════════════════════════════════════════════════════╣')}\n`;
  output += `${chalk.cyan('║')} ID: ${chalk.white.bold((event.event_id || '?').padEnd(49))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('║')} Time: ${chalk.white(tsStr.padEnd(49))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('║')} Severity: ${chalk[severityColor].bold(severity.padEnd(49))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('║')} Type: ${chalk.white((event.type || '?').padEnd(49))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('║')} Service: ${chalk.white((event.service_id || '?').padEnd(49))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('║')} Status: ${chalk.green.bold((event.status || '?').padEnd(49))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('╠═══════════════════════════════════════════════════════════════╣')}\n`;
  output += `${chalk.cyan('║')} Snapshot Data:                                                ${chalk.cyan('║')}\n`;

  if (event.snapshot) {
    const snapshotStr = JSON.stringify(event.snapshot, null, 2);
    for (const line of snapshotStr.split('\n').slice(0, 20)) {
      output += `${chalk.cyan('║')} ${chalk.dim(truncate(line, 61).padEnd(61))} ${chalk.cyan('║')}\n`;
    }
  }

  output += `${chalk.cyan('╚═══════════════════════════════════════════════════════════════╝')}\n`;

  return output;
}

export function printAnalyze(response: IpcResponse, width: number = 63): string {
  if (!response.success) {
    const error = response.data?.error || 'Unknown error';
    return `${chalk.red.bold('❌ Error:')} ${error}`;
  }

  const data = response.data;
  const eventId = data.event_id || '?';

  if (!data.suggestion) {
    return chalk.red('❌ No suggestion data found in response');
  }

  let output = `\n${chalk.cyan('╔═══════════════════════════════════════════════════════════════╗')}\n`;
  output += `${chalk.cyan.bold('║')}              LLM Analysis for Event: ${chalk.white.bold(truncate(eventId, 24).padEnd(24))} ${chalk.cyan('║')}\n`;
  output += `${chalk.cyan('╠═══════════════════════════════════════════════════════════════╣')}\n`;

  if (data.suggestion.analysis) {
    const analysisText = typeof data.suggestion.analysis === 'string' 
      ? data.suggestion.analysis 
      : JSON.stringify(data.suggestion.analysis);
    
    output += `${chalk.cyan('║')} Analysis:                                                  ${chalk.cyan('║')}\n`;
    for (const line of analysisText.split('\n')) {
      const wrapped = wrapText(line, 59);
      for (const wline of wrapped) {
        output += `${chalk.cyan('║')} ${chalk.white(wline.padEnd(61))} ${chalk.cyan('║')}\n`;
      }
    }
  }

  if (data.suggestion.model) {
    output += `${chalk.cyan('╠═══════════════════════════════════════════════════════════════╣')}\n`;
    output += `${chalk.cyan('║')} Model: ${chalk.white((data.suggestion.model || 'unknown').padEnd(49))} ${chalk.cyan('║')}\n`;
  }

  if (data.suggestion.source) {
    output += `${chalk.cyan('║')} Source: ${chalk.white((data.suggestion.source || 'unknown').padEnd(49))} ${chalk.cyan('║')}\n`;
  }

  output += `${chalk.cyan('╚═══════════════════════════════════════════════════════════════╝')}\n`;

  return output;
}

export function printConfig(response: IpcResponse, width: number = 63): string {
  if (!response.success) {
    const error = response.data?.error || 'Unknown error';
    return `${chalk.red.bold('❌ Error:')} ${error}`;
  }

  const thresholds = response.data?.thresholds;
  if (!thresholds) {
    return `${chalk.yellow('⚠️  Thresholds not available')}`;
  }

  const boxWidth = Math.max(width - 2, 59);
  const contentWidth = boxWidth - 2;
  
  const stripAnsi = (str: string) => str.replace(/\u001b\[[0-9;]*m/g, '');
  const makeLine = (leftText: string, rightText: string = '') => {
    const leftLen = stripAnsi(leftText).length;
    const rightLen = stripAnsi(rightText).length;
    const padding = Math.max(0, contentWidth - leftLen - rightLen);
    return `${chalk.cyan('║')} ${leftText}${' '.repeat(padding)}${rightText} ${chalk.cyan('║')}`;
  };

  const title = 'Configuration Thresholds';
  const titlePadding = Math.floor((boxWidth - title.length) / 2);

  return `
${chalk.cyan('╔' + '═'.repeat(boxWidth) + '╗')}
${chalk.cyan.bold('║' + ' '.repeat(titlePadding) + title + ' '.repeat(boxWidth - title.length - titlePadding) + '║')}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('CPU Thresholds:', '')}
${makeLine('  Warning:', chalk.yellow(`${thresholds.cpu_warning}%`))}
${makeLine('  Critical:', chalk.red(`${thresholds.cpu_critical}%`))}
${makeLine('  Sustained Count:', chalk.white(`${thresholds.cpu_sustained_count} checks`))}
${chalk.cyan('╠' + '═'.repeat(boxWidth) + '╣')}
${makeLine('Memory Thresholds:', '')}
${makeLine('  Warning:', chalk.yellow(`${thresholds.memory_warning}%`))}
${makeLine('  Critical:', chalk.red(`${thresholds.memory_critical}%`))}
${chalk.cyan('╚' + '═'.repeat(boxWidth) + '╝')}
${chalk.dim('\nNote: Edit config/default.toml to change these values. Restart the agent to apply changes.')}

`;
}

export function showHelp(): string {
  return `
${chalk.cyan.bold('Available Commands:')}
  ${chalk.cyan('/status')} ${chalk.white('- Show live agent status and metrics (updates every 2s)')}
  ${chalk.cyan('/stop')} ${chalk.white('- Stop live status updates')}
  ${chalk.cyan('/config')} ${chalk.white('- Show current threshold configuration')}
  ${chalk.cyan('/list [limit]')} ${chalk.white('- List recent events (default: 20)')}
  ${chalk.cyan('/show <event_id>')} ${chalk.white('- Show event details')}
  ${chalk.cyan('/analyze <event_id>')} ${chalk.white('- Get LLM analysis for event')}
  ${chalk.cyan('/help')} ${chalk.white('- Show this help message')}
  ${chalk.cyan('/exit|/quit')} ${chalk.white('- Exit the CLI')}

`;
}

function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

function formatTimestamp(ts: number): string {
  const date = new Date(ts * 1000);
  return date.toISOString().replace('T', ' ').slice(0, 19);
}

function truncate(s: string, maxLen: number): string {
  if (s.length <= maxLen) {
    return s.padEnd(maxLen);
  } else {
    return s.slice(0, maxLen - 2) + '..';
  }
}

function wrapText(text: string, maxWidth: number): string[] {
  const words = text.split(/\s+/);
  const result: string[] = [];
  let currentLine = '';

  for (const word of words) {
    if (currentLine === '') {
      currentLine = word;
    } else if (currentLine.length + word.length + 1 <= maxWidth) {
      currentLine += ' ' + word;
    } else {
      result.push(currentLine);
      currentLine = word;
    }
  }

  if (currentLine) {
    result.push(currentLine);
  }

  return result.length > 0 ? result : [''];
}

