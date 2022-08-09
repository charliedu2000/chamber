use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{self},
    str::from_utf8,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Editing,
    Stopped,
}

struct Client {
    username: String,
    input_mode: InputMode,
    received_messages: Vec<String>,
    input_buffer: String,
}
impl Default for Client {
    fn default() -> Client {
        Client {
            username: String::from("temp_user"),
            input_mode: InputMode::Editing, // 自动进入编辑模式
            received_messages: vec![],
            input_buffer: String::default(),
        }
    }
}

pub fn ui_init() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let client = Client::default();
    let res = run_app(&mut terminal, client);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// 响应事件
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut client: Client) -> io::Result<()> {
    let mut cursor_offset: u16 = 0;
    loop {
        terminal.draw(|frame| ui(frame, &client, &cursor_offset))?;

        if let Event::Key(key) = event::read()? {
            match client.input_mode {
                InputMode::Editing => {
                    match key.code {
                        KeyCode::Enter => {
                            // should send msg
                            client
                                .received_messages
                                .push(client.input_buffer.drain(..).collect());
                            cursor_offset = 0;
                        }
                        KeyCode::Char(ch) => {
                            client.input_buffer.push(ch);
                        }
                        KeyCode::Backspace => {
                            client.input_buffer.pop();
                        }
                        KeyCode::Left => {
                            cursor_offset += if cursor_offset < client.input_buffer.width() as u16 {
                                1
                            } else {
                                0
                            }
                        }
                        KeyCode::Right => cursor_offset -= if cursor_offset > 0 { 1 } else { 0 },
                        KeyCode::Esc => {
                            // should set input_mode to STOPPED
                            // client.input_mode = InputMode::Stopped;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                InputMode::Stopped => {}
            }
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, client: &Client, cursor_offset: &u16) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(size);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chunks[0]);

    let msg_block = Block::default()
        .borders(Borders::ALL)
        .title("Chamber Message Window")
        .title_alignment(Alignment::Left);
    let last_msg_received = Paragraph::new::<String>(
        client
            .received_messages
            .last()
            .unwrap_or(&"".to_string())
            .clone(),
    )
    .wrap(Wrap { trim: false })
    .block(msg_block);
    frame.render_widget(last_msg_received, left_chunks[0]);

    let online_clents = Block::default()
        .borders(Borders::ALL)
        .title("Online clients")
        .title_alignment(Alignment::Left);
    frame.render_widget(online_clents, chunks[1]);

    let editor_title =
        "Press <Enter> to send, cursor offset: ".to_string() + &cursor_offset.to_string();
    let editor_block = Block::default()
        .borders(Borders::ALL)
        // .title("Press <Enter> to send, cursor offset: ")
        .title(editor_title)
        .title_alignment(Alignment::Left);
    let msg_in_editor = Paragraph::new(client.input_buffer.as_ref())
        .wrap(Wrap { trim: true })
        .block(editor_block);
    let editor_width = left_chunks[1].width - 2;
    let msg_width = client.input_buffer.width() as u16;
    // set cursor
    frame.set_cursor(
        left_chunks[1].x + ((msg_width - cursor_offset) % editor_width) + 1,
        left_chunks[1].y + ((msg_width - cursor_offset) / editor_width) + 1,
    );
    frame.render_widget(msg_in_editor, left_chunks[1]);
}
