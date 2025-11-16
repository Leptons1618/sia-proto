use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::io::{self, Write};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::MoveTo,
    execute,
};
use colored::*;

#[derive(Serialize)]
struct IpcRequest {
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
}

#[derive(Deserialize)]
struct IpcResponse {
    success: bool,
    data: serde_json::Value,
}

const COMMANDS: &[&str] = &["status", "live", "list", "show", "analyze", "help", "exit", "quit"];

fn show_greeting() {
    println!("\n");
    println!("{}", "  ███████╗██╗ █████╗ ".bright_cyan());
    println!("{}", "  ██╔════╝██║██╔══██╗".bright_cyan());
    println!("{}", "  ███████╗██║███████║".bright_cyan());
    println!("{}", "  ╚════██║██║██╔══██║".bright_cyan());
    println!("{}", "  ███████║██║██║  ██║".bright_cyan());
    println!("{}", "  ╚══════╝╚═╝╚═╝  ╚═╝".bright_cyan());
    println!("{}", "     CLI - System Insight Agent".bright_cyan().dimmed());
    println!("\n{}", "  Type /help for available commands".bright_white().dimmed());
    println!();
}

fn show_input_box(input: &str, suggestions: &[String], selected: usize) {
    // Box width: 59 characters inside (total 61 with borders)
    let box_width: usize = 59;
    let prompt = "▶ ";
    let prompt_len: usize = 2; // "▶ " is 2 characters
    
    // Calculate remaining space (box_width - prompt_len - input.len())
    let remaining = box_width.saturating_sub(prompt_len + input.len());
    
    // Draw input box
    print!("{}", "┌─────────────────────────────────────────────────────────────┐\n".bright_cyan());
    print!("{}", "│ ".bright_cyan());
    print!("{}", prompt.bright_cyan().bold());
    print!("{}", input.bright_white());
    // Fill remaining space
    for _ in 0..remaining {
        print!(" ");
    }
    print!("{}", "│\n".bright_cyan());
    print!("{}", "└─────────────────────────────────────────────────────────────┘\n".bright_cyan());
    
    // Show suggestions if any
    if !suggestions.is_empty() {
        print!("{}", "  Available commands:\n".dimmed());
        for (i, sug) in suggestions.iter().enumerate() {
            if i == selected {
                print!("{}", format!("  ▶ {}\n", sug).bright_cyan().bold());
            } else {
                print!("{}", format!("    {}\n", sug).dimmed());
            }
        }
    }
    
    io::stdout().flush().ok();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check if running in command mode (with args)
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        // Command mode - run single command and exit
        return run_command_mode(&args[1..]).await;
    }
    
    // Interactive mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    
    // Clear screen and show greeting
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    show_greeting();
    stdout.flush()?;
    
    let mut input = String::new();
    let mut suggestions: Vec<String> = Vec::new();
    let mut selected_suggestion = 0;
    
    loop {
        // Clear the input area (from line 9 onwards) before redrawing
        execute!(stdout, MoveTo(0, 9), Clear(ClearType::FromCursorDown))?;
        show_input_box(&input, &suggestions, selected_suggestion);
        
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    match code {
                        KeyCode::Enter => {
                            let cmd = if !suggestions.is_empty() && selected_suggestion < suggestions.len() {
                                suggestions[selected_suggestion].clone()
                            } else {
                                input.clone()
                            };
                            
                            let cmd = cmd.trim();
                            if !cmd.is_empty() {
                                // Clear input area and below
                                execute!(stdout, MoveTo(0, 9), Clear(ClearType::FromCursorDown))?;
                                stdout.flush()?;
                                
                                // Execute command
                                if let Err(e) = handle_command(cmd.trim_start_matches('/')).await {
                                    println!("{} {}", "❌ Error:".red().bold(), e);
                                }
                                
                                println!();
                            }
                            
                            input.clear();
                            suggestions.clear();
                            selected_suggestion = 0;
                        }
                        KeyCode::Char(c) => {
                            if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                break;
                            }
                            if c == 'q' && input.is_empty() {
                                break;
                            }
                            input.push(c);
                            
                            // Update suggestions when "/" is typed
                            if input.starts_with("/") {
                                let query = &input[1..];
                                suggestions = COMMANDS
                                    .iter()
                                    .filter(|cmd| cmd.starts_with(query))
                                    .map(|s| format!("/{}", s))
                                    .collect();
                                selected_suggestion = 0;
                            } else {
                                suggestions.clear();
                                selected_suggestion = 0;
                            }
                        }
                        KeyCode::Backspace => {
                            if !input.is_empty() {
                                input.pop();
                                
                                // Update suggestions
                                if input.starts_with("/") {
                                    let query = &input[1..];
                                    suggestions = COMMANDS
                                        .iter()
                                        .filter(|cmd| cmd.starts_with(query))
                                        .map(|s| format!("/{}", s))
                                        .collect();
                                    selected_suggestion = 0;
                                } else {
                                    suggestions.clear();
                                    selected_suggestion = 0;
                                }
                            }
                        }
                        KeyCode::Up => {
                            if !suggestions.is_empty() {
                                selected_suggestion = selected_suggestion.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if !suggestions.is_empty() {
                                selected_suggestion = (selected_suggestion + 1).min(suggestions.len().saturating_sub(1));
                            }
                        }
                        KeyCode::Tab => {
                            if !suggestions.is_empty() && selected_suggestion < suggestions.len() {
                                input = suggestions[selected_suggestion].clone();
                                suggestions.clear();
                                selected_suggestion = 0;
                            }
                        }
                        KeyCode::Esc => {
                            suggestions.clear();
                            selected_suggestion = 0;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    
    disable_raw_mode()?;
    println!("\n{}", "Goodbye!".bright_green());
    
    Ok(())
}

async fn run_command_mode(args: &[String]) -> Result<()> {
    if args.is_empty() {
        show_greeting();
        return Ok(());
    }
    
    let cmd = args[0].as_str();
    match cmd {
        "status" | "/status" => {
            let response = send_request("status", None, None).await?;
            print_status(response);
        }
        "live" | "/live" => {
            println!("{}", "Live mode not implemented in command mode".yellow());
        }
        "list" | "/list" => {
            let limit = args.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(20);
            let response = send_request("list", Some(limit), None).await?;
            print_list(response);
        }
        "show" | "/show" => {
            if let Some(event_id) = args.get(1) {
                let response = send_request("show", None, Some(event_id.clone())).await?;
                print_show(response);
            } else {
                eprintln!("{}", "❌ Usage: show <event_id>".red());
            }
        }
        "analyze" | "/analyze" => {
            if let Some(event_id) = args.get(1) {
                let response = send_request("analyze", None, Some(event_id.clone())).await?;
                print_analyze(response);
            } else {
                eprintln!("{}", "❌ Usage: analyze <event_id>".red());
            }
        }
        "help" | "/help" | "-h" | "--help" => {
            show_help();
        }
        _ => {
            eprintln!("{} {}", "❌ Unknown command:".red(), cmd);
            eprintln!("{}", "Run 'sia-cli help' for usage information".dimmed());
        }
    }
    
    Ok(())
}

async fn handle_command(cmd: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    let cmd = parts[0];
    match cmd {
        "status" => {
            let response = send_request("status", None, None).await?;
            print_status(response);
        }
        "live" => {
            println!("{}", "Live mode - press Ctrl+C to exit".yellow());
            // Simple live loop
            let mut stdout = io::stdout();
            loop {
                let response = send_request("status", None, None).await?;
                execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                print_status(response);
                stdout.flush()?;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
        "list" => {
            let limit = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(20);
            let response = send_request("list", Some(limit), None).await?;
            print_list(response);
        }
        "show" => {
            if let Some(event_id) = parts.get(1) {
                let response = send_request("show", None, Some((*event_id).to_string())).await?;
                print_show(response);
            } else {
                println!("{}", "❌ Usage: show <event_id>".red());
            }
        }
        "analyze" => {
            if let Some(event_id) = parts.get(1) {
                println!("{}", "🤖 Analyzing event with LLM...".bright_yellow());
                let response = send_request("analyze", None, Some((*event_id).to_string())).await?;
                print_analyze(response);
            } else {
                println!("{}", "❌ Usage: analyze <event_id>".red());
            }
        }
        "help" => {
            show_help();
        }
        "exit" | "quit" | "q" | "bye" => {
            println!("{}", "Goodbye!".bright_green());
            std::process::exit(0);
        }
        _ => {
            println!("{} {}", "❌ Unknown command:".red().bold(), cmd);
            println!("{}", "Type /help for available commands".dimmed());
        }
    }
    
    Ok(())
}

fn show_help() {
    println!("\n{}", "Available Commands:".bright_cyan().bold());
    println!("{} {}", "  /status", "- Show agent status and metrics".bright_white());
    println!("{} {}", "  /live", "- Live status dashboard".bright_white());
    println!("{} {}", "  /list [limit]", "- List recent events (default: 20)".bright_white());
    println!("{} {}", "  /show <event_id>", "- Show event details".bright_white());
    println!("{} {}", "  /analyze <event_id>", "- Get LLM analysis for event".bright_white());
    println!("{} {}", "  /help", "- Show this help message".bright_white());
    println!("{} {}", "  /exit|/quit", "- Exit the CLI".bright_white());
    println!();
}

async fn send_request(method: &str, limit: Option<i32>, event_id: Option<String>) -> Result<IpcResponse> {
    let socket_path = if std::path::Path::new("/run/sia/sia.sock").exists() {
        "/run/sia/sia.sock"
    } else if std::path::Path::new("/tmp/sia.sock").exists() {
        "/tmp/sia.sock"
    } else {
        return Err(anyhow::anyhow!("No SIA socket found. Is the agent running?"));
    };
    let mut stream = UnixStream::connect(socket_path).await?;
    
    let request = IpcRequest {
        method: method.to_string(),
        limit,
        event_id,
    };
    
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.shutdown().await?;
    
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    
    let response: IpcResponse = serde_json::from_slice(&buffer)?;
    Ok(response)
}

fn print_status(response: IpcResponse) {
    if !response.success {
        println!("{} {}", "❌ Error:".red().bold(), response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let data = &response.data;
    let uptime = data["uptime_seconds"].as_u64().unwrap_or(0);
    let uptime_str = format_uptime(uptime);
    
    let cpu_usage = data["metrics"]["cpu_usage"].as_f64().unwrap_or(0.0) as f32;
    let mem_used = data["metrics"]["memory_used_mb"].as_u64().unwrap_or(0);
    let mem_total = data["metrics"]["memory_total_mb"].as_u64().unwrap_or(0);
    let mem_percent = data["metrics"]["memory_percent"].as_f64().unwrap_or(0.0) as f32;
    
    let cpu_color: colored::Color = if cpu_usage > 80.0 { colored::Color::Red } else if cpu_usage > 60.0 { colored::Color::Yellow } else { colored::Color::Green };
    let mem_color: colored::Color = if mem_percent > 85.0 { colored::Color::Red } else if mem_percent > 70.0 { colored::Color::Yellow } else { colored::Color::Green };
    
    println!("\n{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                    SIA Agent Status                           ║".bright_cyan().bold());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{} {:49} {}", "║ Uptime:".bright_cyan(), uptime_str.bright_white(), "║".bright_cyan());
    println!("{} {:49} {}", "║ Status:".bright_cyan(), data["status"].as_str().unwrap_or("unknown").bright_green().bold(), "║".bright_cyan());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{}", "║ System Metrics:                                              ║".bright_cyan());
    println!("{} {:47.1}% {}", "║   CPU Usage:".bright_cyan(), cpu_usage.to_string().color(cpu_color).bold(), "║".bright_cyan());
    println!("{} {:47} {}", "║   Memory:".bright_cyan(), format!("{:.1}% ({} MB / {} MB)", mem_percent, mem_used, mem_total).color(mem_color).bold(), "║".bright_cyan());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{}", "║ Collectors:                                                   ║".bright_cyan());
    println!("{} {:49} {}", "║   CPU:".bright_cyan(), format!("✓ {}", data["collectors"]["cpu"].as_str().unwrap_or("unknown")).bright_green(), "║".bright_cyan());
    println!("{} {:49} {}", "║   Memory:".bright_cyan(), format!("✓ {}", data["collectors"]["memory"].as_str().unwrap_or("unknown")).bright_green(), "║".bright_cyan());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{}", "║ Events (open):                                                ║".bright_cyan());
    println!("{} {:49} {}", "║   Critical:".bright_cyan(), data["events"]["critical"].as_i64().unwrap_or(0).to_string().bright_red().bold(), "║".bright_cyan());
    println!("{} {:49} {}", "║   Warning:".bright_cyan(), data["events"]["warning"].as_i64().unwrap_or(0).to_string().bright_yellow().bold(), "║".bright_cyan());
    println!("{} {:49} {}", "║   Info:".bright_cyan(), data["events"]["info"].as_i64().unwrap_or(0).to_string().bright_blue().bold(), "║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝\n".bright_cyan());
}

fn print_list(response: IpcResponse) {
    if !response.success {
        println!("{} {}", "❌ Error:".red().bold(), response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let empty_vec = vec![];
    let events = response.data["events"].as_array().unwrap_or(&empty_vec);
    
    if events.is_empty() {
        println!("\n{}", "📭 No events found.\n".dimmed());
        return;
    }
    
    println!("\n{}", "┌──────────────────────────┬────────────────────────┬──────────┬───────────────┬────────┐".bright_cyan());
    println!("{}", "│ Event ID                 │ Timestamp              │ Severity │ Type          │ Status │".bright_cyan().bold());
    println!("{}", "├──────────────────────────┼────────────────────────┼──────────┼───────────────┼────────┤".bright_cyan());
    
    for event in events {
        let event_id = event["event_id"].as_str().unwrap_or("?");
        let ts = event["ts"].as_i64().unwrap_or(0);
        let ts_str = format_timestamp(ts);
        let severity = event["severity"].as_str().unwrap_or("?");
        let type_ = event["type"].as_str().unwrap_or("?");
        let status = event["status"].as_str().unwrap_or("?");
        
        let severity_color: colored::Color = match severity {
            "CRITICAL" => colored::Color::Red,
            "WARNING" => colored::Color::Yellow,
            "INFO" => colored::Color::Blue,
            _ => colored::Color::White,
        };
        
        println!("│ {:24} │ {:22} │ {:8} │ {:13} │ {:6} │",
            truncate(event_id, 24).bright_white(),
            ts_str.dimmed(),
            severity.color(severity_color).bold(),
            truncate(type_, 13).bright_white(),
            status.bright_green()
        );
    }
    
    println!("{}", "└──────────────────────────┴────────────────────────┴──────────┴───────────────┴────────┘\n".bright_cyan());
}

fn print_show(response: IpcResponse) {
    if !response.success {
        println!("{} {}", "❌ Error:".red().bold(), response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let event = &response.data;
    let ts = event["ts"].as_i64().unwrap_or(0);
    let ts_str = format_timestamp(ts);
    
    let severity = event["severity"].as_str().unwrap_or("?");
    let severity_color: colored::Color = match severity {
        "CRITICAL" => colored::Color::Red,
        "WARNING" => colored::Color::Yellow,
        "INFO" => colored::Color::Blue,
        _ => colored::Color::White,
    };
    
    println!("\n{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                      Event Details                            ║".bright_cyan().bold());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{} {:49} {}", "║ ID:".bright_cyan(), event["event_id"].as_str().unwrap_or("?").bright_white().bold(), "║".bright_cyan());
    println!("{} {:49} {}", "║ Time:".bright_cyan(), ts_str.bright_white(), "║".bright_cyan());
    println!("{} {:49} {}", "║ Severity:".bright_cyan(), severity.color(severity_color).bold(), "║".bright_cyan());
    println!("{} {:49} {}", "║ Type:".bright_cyan(), event["type"].as_str().unwrap_or("?").bright_white(), "║".bright_cyan());
    println!("{} {:49} {}", "║ Service:".bright_cyan(), event["service_id"].as_str().unwrap_or("?").bright_white(), "║".bright_cyan());
    println!("{} {:49} {}", "║ Status:".bright_cyan(), event["status"].as_str().unwrap_or("?").bright_green().bold(), "║".bright_cyan());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{}", "║ Snapshot Data:                                                ║".bright_cyan());
    
    if let Some(snapshot) = event.get("snapshot") {
        let snapshot_str = serde_json::to_string_pretty(snapshot).unwrap_or_default();
        for line in snapshot_str.lines().take(20) {
            println!("{} {:61} {}", "║".bright_cyan(), truncate(line, 61).dimmed(), "║".bright_cyan());
        }
    }
    
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝\n".bright_cyan());
}

fn print_analyze(response: IpcResponse) {
    if !response.success {
        println!("{} {}", "❌ Error:".red().bold(), response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let data = &response.data;
    let event_id = data["event_id"].as_str().unwrap_or("?");
    
    if let Some(suggestion) = data.get("suggestion") {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{} {:24} {}", "║              LLM Analysis for Event:".bright_cyan().bold(), truncate(event_id, 24).bright_white().bold(), "║".bright_cyan());
        println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
        
        if let Some(analysis) = suggestion.get("analysis") {
            if let Some(analysis_text) = analysis.as_str() {
                println!("{}", "║ Analysis:                                                  ║".bright_cyan());
                for line in analysis_text.lines() {
                    let wrapped = wrap_text(line, 59);
                    for wline in wrapped {
                        println!("{} {:61} {}", "║".bright_cyan(), wline.bright_white(), "║".bright_cyan());
                    }
                }
            }
        }
        
        if let Some(model) = suggestion.get("model") {
            println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
            println!("{} {:49} {}", "║ Model:".bright_cyan(), model.as_str().unwrap_or("unknown").bright_white(), "║".bright_cyan());
        }
        
        if let Some(source) = suggestion.get("source") {
            println!("{} {:49} {}", "║ Source:".bright_cyan(), source.as_str().unwrap_or("unknown").bright_white(), "║".bright_cyan());
        }
        
        println!("{}", "╚═══════════════════════════════════════════════════════════════╝\n".bright_cyan());
    } else {
        println!("{}", "❌ No suggestion data found in response".red());
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut result = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut current_line = String::new();
    
    for word in words {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() + 1 <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push(current_line.clone());
            current_line = word.to_string();
        }
    }
    
    if !current_line.is_empty() {
        result.push(current_line);
    }
    
    if result.is_empty() {
        result.push(String::new());
    }
    
    result
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn format_timestamp(ts: i64) -> String {
    use chrono::{Utc, TimeZone};
    
    match Utc.timestamp_opt(ts, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => "Invalid timestamp".to_string(),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}..", &s[..max_len-2])
    }
}
