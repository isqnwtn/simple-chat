use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpStream, sync::mpsc
};

enum Event {
    Input(String),
    ServerMsg(String),
    Tick,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Channel between UI and network
    let (tx, mut rx) = mpsc::channel::<Event>(100);

    // Connect to server
    let addr: SocketAddr = "127.0.0.1:7878".parse().unwrap();
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let (reader, mut writer) = split(stream);
    let mut reader = BufReader::new(reader);

    // Task: read from server
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_clone.send(Event::ServerMsg(line)).await;
        }
    });

    // Task: poll keyboard
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Enter => {
                            // handled in main loop
                            let _ = tx_clone.send(Event::Input("\n".to_string())).await;
                        }
                        KeyCode::Char(c) => {
                            let _ = tx_clone.send(Event::Input(c.to_string())).await;
                        }
                        KeyCode::Backspace => {
                            let _ = tx_clone.send(Event::Input("\x08".to_string())).await;
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    });

    let mut messages: Vec<String> = vec![];
    let mut input = String::new();

    // Main UI loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
                .split(f.size());

            let chat_text: Vec<Spans> =
                messages.iter().map(|m| Spans::from(m.clone())).collect();
            let chat_box = Paragraph::new(chat_text)
                .block(Block::default().borders(Borders::ALL).title("Chat"));
            f.render_widget(chat_box, chunks[0]);

            let input_box = Paragraph::new(input.clone())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input_box, chunks[1]);
        })?;

        // Handle events
        if let Some(event) = rx.recv().await {
            match event {
                Event::ServerMsg(msg) => {
                    messages.push(format!("Server: {}", msg));
                }
                Event::Input(s) => {
                    if s == "\n" {
                        if !input.is_empty() {
                            // Send to server
                            writer.write_all(input.as_bytes()).await.unwrap();
                            writer.write_all(b"\n").await.unwrap();
                            messages.push(format!("Me: {}", input));
                            input.clear();
                        }
                    } else if s == "\x08" {
                        input.pop();
                    } else {
                        input.push_str(&s);
                    }
                }
                Event::Tick => {}
            }
        }
    }

    // Cleanup
    // (unreachable in this infinite loop unless Esc handling is added properly)
    // disable_raw_mode()?;
    // execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    // terminal.show_cursor()?;
    // Ok(())
}
