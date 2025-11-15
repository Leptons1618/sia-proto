use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::io::{self, Write};
use std::process::Command;
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
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

const COMMANDS: &[&str] = &["status", "live", "list", "show", "analyze", "help", "exit", "quit", "bye", "clear"];

#[derive(Clone)]
struct StatusData {
    uptime: u64,
    status: String,
    cpu_usage: f32,
    mem_used: u64,
    mem_total: u64,
    mem_percent: f32,
    cpu_collector: String,
    mem_collector: String,
    critical: i64,
    warning: i64,
    info: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check if running in interactive mode (no args) or command mode
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        // Command mode - run single command and exit
        return run_command_mode(&args[1..]).await;
    }
    
    // Interactive mode with TUI
    show_welcome();
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    
    let mut input = String::new();
    let mut suggestions: Vec<String> = Vec::new();
    let mut selected_suggestion = 0;
    
    loop {
        // Clear suggestions area and redraw
        execute!(stdout, MoveTo(0, 20))?;
        execute!(stdout, Clear(ClearType::FromCursorDown))?;
        
        // Draw prompt and input
        execute!(stdout, MoveTo(0, 19))?;
        print!("{}", "sia> ".bright_cyan().bold());
        print!("{}", input);
        io::stdout().flush()?;
        
        // Show suggestions if "/" is typed
        if input.starts_with("/") {
            let query = &input[1..];
            suggestions = COMMANDS
                .iter()
                .filter(|cmd| cmd.starts_with(query))
                .map(|s| format!("/{}", s))
                .collect();
            
            if !suggestions.is_empty() {
                execute!(stdout, MoveTo(0, 20))?;
                println!("{}", "  Suggestions:".dimmed());
                for (i, sug) in suggestions.iter().enumerate() {
                    let style = if i == selected_suggestion {
                        sug.bright_cyan().bold().on_black()
                    } else {
                        sug.dimmed()
                    };
                    println!("  {}", style);
                }
            }
        } else {
            suggestions.clear();
        }
        
        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    match code {
                        KeyCode::Enter => {
                            if !suggestions.is_empty() && selected_suggestion < suggestions.len() {
                                input = suggestions[selected_suggestion].clone();
                            }
                            
                            execute!(stdout, MoveTo(0, 19))?;
                            execute!(stdout, Clear(ClearType::CurrentLine))?;
                            println!("{} {}", "sia>".bright_cyan().bold(), input);
                            
                            let cmd = input.trim();
                            if !cmd.is_empty() {
                                if let Err(e) = handle_command(cmd).await {
                                    println!("{} {}", "❌ Error:".red().bold(), e);
                                }
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
                                break; // Quick exit on 'q'
                            }
                            input.push(c);
                            selected_suggestion = 0;
                        }
                        KeyCode::Backspace => {
                            input.pop();
                            selected_suggestion = 0;
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
    
    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    println!("\n{}", "Goodbye!".bright_green());
    
    Ok(())
}

async fn run_command_mode(args: &[String]) -> Result<()> {
    if args.is_empty() {
        show_welcome();
        return Ok(());
    }
    
    let cmd = args[0].as_str();
    match cmd {
        "status" | "/status" => {
            let response = send_request("status", None, None).await?;
            print_status(response);
        }
        "live" | "/live" => {
            run_live_status().await?;
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

async fn handle_command(input: &str) -> Result<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    let cmd = parts[0].trim_start_matches('/');
    match cmd {
        "status" => {
            let response = send_request("status", None, None).await?;
            print_status(response);
        }
        "live" => {
            run_live_status().await?;
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
                println!("{}", "❌ Usage: /show <event_id>".red());
            }
        }
        "analyze" => {
            if let Some(event_id) = parts.get(1) {
                println!("{}", "🤖 Analyzing event with LLM...".bright_yellow());
                let response = send_request("analyze", None, Some((*event_id).to_string())).await?;
                print_analyze(response);
            } else {
                println!("{}", "❌ Usage: /analyze <event_id>".red());
            }
        }
        "help" => {
            show_help();
        }
        "exit" | "quit" | "q" | "bye" => {
            println!("{}", "Goodbye!".bright_green());
            std::process::exit(0);
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush()?;
        }
        _ => {
            println!("{} {}", "❌ Unknown command:".red().bold(), cmd);
            println!("{}", "Type /help for available commands".dimmed());
        }
    }
    
    Ok(())
}

fn show_welcome() {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║         SIA - System Insight Agent (Interactive CLI)        ║".bright_cyan().bold());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║  Available Commands:                                         ║".bright_cyan());
    println!("{} {}", "║    /status".bright_cyan(), "              - Show agent status and metrics      ║".bright_cyan());
    println!("{} {}", "║    /live".bright_cyan(), "                - Live status dashboard (updates)     ║".bright_cyan());
    println!("{} {}", "║    /list [limit]".bright_cyan(), "        - List recent events (default: 20)   ║".bright_cyan());
    println!("{} {}", "║    /show <event_id>".bright_cyan(), "     - Show event details                ║".bright_cyan());
    println!("{} {}", "║    /analyze <event_id>".bright_cyan(), "  - Get LLM analysis for event       ║".bright_cyan());
    println!("{} {}", "║    /help".bright_cyan(), "                - Show this help message             ║".bright_cyan());
    println!("{} {}", "║    /exit|/quit|/bye".bright_cyan(), "      - Exit the CLI                      ║".bright_cyan());
    println!("{} {}", "║    /clear".bright_cyan(), "               - Clear the screen                  ║".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║  Tip: Type '/' to see command suggestions                   ║".bright_cyan().dimmed());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝\n".bright_cyan());
}

fn show_help() {
    show_welcome();
}

async fn send_request(method: &str, limit: Option<i32>, event_id: Option<String>) -> Result<IpcResponse> {
    let socket_path = "/run/sia/sia.sock";
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
    
    // Get metrics if available
    let cpu_usage = data["metrics"]["cpu_usage"].as_f64().unwrap_or(0.0) as f32;
    let mem_used = data["metrics"]["memory_used_mb"].as_u64().unwrap_or(0);
    let mem_total = data["metrics"]["memory_total_mb"].as_u64().unwrap_or(0);
    let mem_percent = data["metrics"]["memory_percent"].as_f64().unwrap_or(0.0) as f32;
    
    // Color coding for CPU/Memory
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

async fn run_live_status() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    
    let mut menu_selected = 0;
    let menu_items = vec!["Start Service", "Stop Service", "Restart Service", "View Logs", "Back"];
    let mut last_status: Option<StatusData> = None;
    let mut update_counter = 0;
    
    loop {
        // Fetch current status
        let response = match send_request("status", None, None).await {
            Ok(r) => r,
            Err(_) => {
                // Service might be down
                let error_status = StatusData {
                    uptime: 0,
                    status: "disconnected".to_string(),
                    cpu_usage: 0.0,
                    mem_used: 0,
                    mem_total: 0,
                    mem_percent: 0.0,
                    cpu_collector: "unknown".to_string(),
                    mem_collector: "unknown".to_string(),
                    critical: 0,
                    warning: 0,
                    info: 0,
                };
                render_live_status(&error_status, &menu_items, menu_selected, update_counter).await?;
                
                // Check for input
                if event::poll(Duration::from_millis(100))? {
                    match handle_live_input(&mut menu_selected, &menu_items).await? {
                        LiveAction::Exit => break,
                        LiveAction::Select => {
                            handle_menu_action(menu_selected, &menu_items).await?;
                        }
                        LiveAction::None => {}
                    }
                }
                sleep(Duration::from_millis(500)).await;
                continue;
            }
        };
        
        if response.success {
            let status = parse_status_data(&response.data);
            
            // Only update if data changed
            let should_update = last_status.as_ref().map_or(true, |old| {
                old.cpu_usage != status.cpu_usage ||
                old.mem_percent != status.mem_percent ||
                old.critical != status.critical ||
                old.warning != status.warning ||
                old.info != status.info ||
                old.uptime != status.uptime
            });
            
            if should_update || update_counter % 10 == 0 {
                render_live_status(&status, &menu_items, menu_selected, update_counter).await?;
                last_status = Some(status);
            }
        }
        
        update_counter += 1;
        
        // Check for input
        if event::poll(Duration::from_millis(100))? {
            match handle_live_input(&mut menu_selected, &menu_items).await? {
                LiveAction::Exit => break,
                LiveAction::Select => {
                    handle_menu_action(menu_selected, &menu_items).await?;
                }
                LiveAction::None => {}
            }
        }
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    
    Ok(())
}

enum LiveAction {
    Exit,
    Select,
    None,
}

async fn handle_live_input(selected: &mut usize, menu_items: &[&str]) -> Result<LiveAction> {
    match event::read()? {
        Event::Key(KeyEvent { code, modifiers, .. }) => {
            match code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(LiveAction::Exit);
                }
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(LiveAction::Exit);
                }
                KeyCode::Up => {
                    *selected = selected.saturating_sub(1);
                }
                KeyCode::Down => {
                    *selected = (*selected + 1).min(menu_items.len().saturating_sub(1));
                }
                KeyCode::Enter => {
                    return Ok(LiveAction::Select);
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(LiveAction::None)
}

async fn handle_menu_action(selected: usize, menu_items: &[&str]) -> Result<()> {
    if selected >= menu_items.len() {
        return Ok(());
    }
    
    let action = menu_items[selected];
    let mut stdout = io::stdout();
    
    match action {
        "Start Service" => {
            execute!(stdout, MoveTo(0, 25))?;
            println!("{}", "🔄 Starting service...".bright_yellow());
            io::stdout().flush()?;
            
            let output = Command::new("systemctl")
                .args(&["start", "sia-agent"])
                .output()?;
            
            execute!(stdout, MoveTo(0, 25))?;
            execute!(stdout, Clear(ClearType::CurrentLine))?;
            
            if output.status.success() {
                println!("{}", "✅ Service started successfully".bright_green());
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("{} {}", "❌ Failed to start service:".red(), error);
            }
            io::stdout().flush()?;
            sleep(Duration::from_secs(2)).await;
        }
        "Stop Service" => {
            execute!(stdout, MoveTo(0, 25))?;
            println!("{}", "🛑 Stopping service...".bright_yellow());
            io::stdout().flush()?;
            
            let output = Command::new("systemctl")
                .args(&["stop", "sia-agent"])
                .output()?;
            
            execute!(stdout, MoveTo(0, 25))?;
            execute!(stdout, Clear(ClearType::CurrentLine))?;
            
            if output.status.success() {
                println!("{}", "✅ Service stopped successfully".bright_green());
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("{} {}", "❌ Failed to stop service:".red(), error);
            }
            io::stdout().flush()?;
            sleep(Duration::from_secs(2)).await;
        }
        "Restart Service" => {
            execute!(stdout, MoveTo(0, 25))?;
            println!("{}", "🔄 Restarting service...".bright_yellow());
            io::stdout().flush()?;
            
            let output = Command::new("systemctl")
                .args(&["restart", "sia-agent"])
                .output()?;
            
            execute!(stdout, MoveTo(0, 25))?;
            execute!(stdout, Clear(ClearType::CurrentLine))?;
            
            if output.status.success() {
                println!("{}", "✅ Service restarted successfully".bright_green());
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("{} {}", "❌ Failed to restart service:".red(), error);
            }
            io::stdout().flush()?;
            sleep(Duration::from_secs(2)).await;
        }
        "View Logs" => {
            execute!(stdout, MoveTo(0, 25))?;
            execute!(stdout, Clear(ClearType::CurrentLine))?;
            println!("{}", "📋 Opening logs (press Ctrl+C to return)...".bright_yellow());
            io::stdout().flush()?;
            sleep(Duration::from_secs(1)).await;
            
            // Execute journalctl in foreground
            execute!(stdout, Show, LeaveAlternateScreen)?;
            disable_raw_mode()?;
            
            let _ = Command::new("journalctl")
                .args(&["-u", "sia-agent", "-n", "50", "-f"])
                .status();
            
            // Re-enter TUI mode
            enable_raw_mode()?;
            execute!(stdout, EnterAlternateScreen, Hide)?;
        }
        "Back" => {
            return Ok(());
        }
        _ => {}
    }
    
    Ok(())
}

// Helper function to get display width without ANSI codes
fn display_width(s: &str) -> usize {
    // Remove ANSI escape sequences for width calculation
    let mut width = 0;
    let mut in_escape = false;
    let mut in_bracket = false;
    
    for ch in s.chars() {
        if in_escape {
            if ch == '[' {
                in_bracket = true;
            } else if in_bracket && (ch == 'm' || ch == 'H' || ch == 'J' || ch == 'K') {
                in_escape = false;
                in_bracket = false;
            } else if !in_bracket && (ch < '0' || ch > '9') && ch != ';' {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            width += ch.len_utf8().min(1); // Count wide chars as 1 for simplicity
        }
    }
    
    width
}

async fn render_live_status(status: &StatusData, menu_items: &[&str], selected: usize, counter: u64) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0))?;
    execute!(stdout, Clear(ClearType::All))?;
    
    let cpu_color: colored::Color = if status.cpu_usage > 80.0 { colored::Color::Red } else if status.cpu_usage > 60.0 { colored::Color::Yellow } else { colored::Color::Green };
    let mem_color: colored::Color = if status.mem_percent > 85.0 { colored::Color::Red } else if status.mem_percent > 70.0 { colored::Color::Yellow } else { colored::Color::Green };
    
    let uptime_str = format_uptime(status.uptime);
    let status_color = if status.status == "running" { colored::Color::Green } else { colored::Color::Red };
    
    // Build the entire screen content as a string
    let mut screen = String::new();
    
    // Helper to add a line
    macro_rules! add_line {
        ($line:expr) => {
            screen.push_str(&format!("{}\n", $line));
        };
    }
    
    // Header
    add_line!("╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    add_line!("║              SIA Live Status Dashboard (Live)                ║".bright_cyan().bold());
    add_line!("╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    
    // Uptime
    let uptime_padding = 49usize.saturating_sub(uptime_str.len());
    add_line!(format!("║ {} {}{} ║", "Uptime:".bright_cyan(), uptime_str.bright_white(), " ".repeat(uptime_padding)));
    
    // Status
    let status_text = status.status.color(status_color).bold();
    let status_padding = 49usize.saturating_sub(status.status.len());
    add_line!(format!("║ {} {}{} ║", "Status:".bright_cyan(), status_text, " ".repeat(status_padding)));
    
    add_line!("╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    add_line!("║ System Metrics:                                              ║".bright_cyan());
    
    // CPU Usage
    let cpu_val_str = format!("{:.1}%", status.cpu_usage);
    let cpu_val = cpu_val_str.color(cpu_color).bold();
    let cpu_padding = 47usize.saturating_sub(cpu_val_str.len());
    add_line!(format!("║ {} {}{} ║", "  CPU Usage:".bright_cyan(), cpu_val, " ".repeat(cpu_padding)));
    
    // Memory
    let mem_val_str = format!("{:.1}% ({} MB / {} MB)", status.mem_percent, status.mem_used, status.mem_total);
    let mem_val = mem_val_str.color(mem_color).bold();
    let mem_padding = 47usize.saturating_sub(mem_val_str.len());
    add_line!(format!("║ {} {}{} ║", "  Memory:".bright_cyan(), mem_val, " ".repeat(mem_padding)));
    
    add_line!("╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    add_line!("║ Collectors:                                                   ║".bright_cyan());
    
    // CPU Collector
    let cpu_collector_str = format!("✓ {}", status.cpu_collector);
    let cpu_collector_val = cpu_collector_str.bright_green();
    let cpu_collector_padding = 49usize.saturating_sub(cpu_collector_str.len());
    add_line!(format!("║ {} {}{} ║", "  CPU:".bright_cyan(), cpu_collector_val, " ".repeat(cpu_collector_padding)));
    
    // Memory Collector
    let mem_collector_str = format!("✓ {}", status.mem_collector);
    let mem_collector_val = mem_collector_str.bright_green();
    let mem_collector_padding = 49usize.saturating_sub(mem_collector_str.len());
    add_line!(format!("║ {} {}{} ║", "  Memory:".bright_cyan(), mem_collector_val, " ".repeat(mem_collector_padding)));
    
    add_line!("╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    add_line!("║ Events (open):                                                ║".bright_cyan());
    
    // Critical
    let critical_str = status.critical.to_string();
    let critical_val = critical_str.bright_red().bold();
    let critical_padding = 49usize.saturating_sub(critical_str.len());
    add_line!(format!("║ {} {}{} ║", "  Critical:".bright_cyan(), critical_val, " ".repeat(critical_padding)));
    
    // Warning
    let warning_str = status.warning.to_string();
    let warning_val = warning_str.bright_yellow().bold();
    let warning_padding = 49usize.saturating_sub(warning_str.len());
    add_line!(format!("║ {} {}{} ║", "  Warning:".bright_cyan(), warning_val, " ".repeat(warning_padding)));
    
    // Info
    let info_str = status.info.to_string();
    let info_val = info_str.bright_blue().bold();
    let info_padding = 49usize.saturating_sub(info_str.len());
    add_line!(format!("║ {} {}{} ║", "  Info:".bright_cyan(), info_val, " ".repeat(info_padding)));
    
    add_line!("╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    add_line!("║ Service Control:                                             ║".bright_cyan());
    
    // Menu items
    for (i, item) in menu_items.iter().enumerate() {
        let prefix = if i == selected { "▶ " } else { "  " };
        let item_text = format!("{}{}", prefix, item);
        let item_width = display_width(&item_text);
        let padding = 47usize.saturating_sub(item_width);
        
        if i == selected {
            let selected_text = format!("{}{}", prefix, item.bright_cyan().bold().on_black());
            add_line!(format!("║{}{:padding$}║", selected_text, ""));
        } else {
            let normal_text = format!("{}{}", prefix, item.bright_white());
            add_line!(format!("║{}{:padding$}║", normal_text, ""));
        }
    }
    
    add_line!("╠═══════════════════════════════════════════════════════════════╣".bright_cyan());
    
    // Updates
    let updates_str = format!("{} (refreshing every 0.5s)", counter);
    let updates_text = updates_str.dimmed();
    let updates_padding = 49usize.saturating_sub(updates_str.len());
    add_line!(format!("║ {} {}{} ║", "Updates:".bright_cyan(), updates_text, " ".repeat(updates_padding)));
    
    add_line!("║ Controls: ↑↓ Navigate | Enter Select | Q/Esc Exit            ║".bright_cyan().dimmed());
    add_line!("╚═══════════════════════════════════════════════════════════════╝".bright_cyan());
    
    // Write everything at once
    write!(stdout, "{}", screen)?;
    stdout.flush()?;
    Ok(())
}

fn parse_status_data(data: &serde_json::Value) -> StatusData {
    StatusData {
        uptime: data["uptime_seconds"].as_u64().unwrap_or(0),
        status: data["status"].as_str().unwrap_or("unknown").to_string(),
        cpu_usage: data["metrics"]["cpu_usage"].as_f64().unwrap_or(0.0) as f32,
        mem_used: data["metrics"]["memory_used_mb"].as_u64().unwrap_or(0),
        mem_total: data["metrics"]["memory_total_mb"].as_u64().unwrap_or(0),
        mem_percent: data["metrics"]["memory_percent"].as_f64().unwrap_or(0.0) as f32,
        cpu_collector: data["collectors"]["cpu"].as_str().unwrap_or("unknown").to_string(),
        mem_collector: data["collectors"]["memory"].as_str().unwrap_or("unknown").to_string(),
        critical: data["events"]["critical"].as_i64().unwrap_or(0),
        warning: data["events"]["warning"].as_i64().unwrap_or(0),
        info: data["events"]["info"].as_i64().unwrap_or(0),
    }
}
