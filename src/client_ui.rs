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
    widgets::{Block, Borders},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::consts::MSG_BUF_SIZE;
use crate::paragraph_chamber::{Paragraph, Wrap};

enum InputMode {
    Editing,
    Stopped,
}

struct App {
    input_mode: InputMode,
    received_messages: Vec<String>,
    input_buffer: String,
    cursor_offset: usize,
    editor_width: usize,
}
impl Default for App {
    fn default() -> App {
        App {
            input_mode: InputMode::Editing, // 自动进入编辑模式
            received_messages: vec![],
            input_buffer: String::default(),
            cursor_offset: 0,
            editor_width: 0,
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
    let app = App::default();
    let res = run_app(&mut terminal, app);

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
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Editing => {
                    match key.code {
                        KeyCode::Enter => {
                            // should send msg
                            app.received_messages
                                .push(app.input_buffer.drain(..).collect());
                            app.cursor_offset = 0;
                        }
                        KeyCode::Char(ch) => {
                            if app.input_buffer.as_bytes().len() < MSG_BUF_SIZE {
                                // app.input_buffer.push(ch);
                                app.input_buffer
                                    .insert(app.input_buffer.len() - app.cursor_offset, ch)
                            }
                        }
                        KeyCode::Backspace => {
                            // client.input_buffer.pop();
                            let cursor_position = app.input_buffer.len() - app.cursor_offset;
                            if cursor_position == app.input_buffer.len() {
                                app.input_buffer.pop();
                            } else if cursor_position > 0 {
                                app.input_buffer.remove(cursor_position - 1);
                            }
                        }
                        KeyCode::Left => {
                            app.cursor_offset += if app.cursor_offset < app.input_buffer.width() {
                                1
                            } else {
                                0
                            }
                        }
                        KeyCode::Right => {
                            app.cursor_offset -= if app.cursor_offset > 0 { 1 } else { 0 };
                        }
                        KeyCode::Up => {
                            // move cursor up
                        }
                        KeyCode::Down => {
                            // move cursor down
                        }
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

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
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
        app.received_messages
            .last()
            .unwrap_or(&"".to_string())
            .clone(),
    )
    .wrap(Wrap {
        trim: true,
        break_words: true,
    })
    .block(msg_block);
    frame.render_widget(last_msg_received, left_chunks[0]);

    let online_clents = Block::default()
        .borders(Borders::ALL)
        .title("Online clients")
        .title_alignment(Alignment::Left);
    frame.render_widget(online_clents, chunks[1]);

    let editor_title = format!(
        "Press <Enter> to send, cursor offset: {}, str len: {}",
        app.cursor_offset,
        app.input_buffer.len()
    );
    let editor_block = Block::default()
        .borders(Borders::ALL)
        // .title("Press <Enter> to send, cursor offset: ")
        .title(editor_title)
        .title_alignment(Alignment::Left);
    let msg_in_editor = Paragraph::new(app.input_buffer.as_ref())
        .wrap(Wrap {
            trim: false,
            break_words: false,
        })
        .block(editor_block);
    app.editor_width = left_chunks[1].width as usize - 2;
    let msg_width = app.input_buffer.width();
    // set cursor
    frame.set_cursor(
        left_chunks[1].x + ((msg_width - app.cursor_offset) % app.editor_width) as u16 + 1,
        left_chunks[1].y + ((msg_width - app.cursor_offset) / app.editor_width) as u16 + 1,
    );
    frame.render_widget(msg_in_editor, left_chunks[1]);
}
