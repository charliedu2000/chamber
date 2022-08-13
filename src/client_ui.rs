use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    consts::MSG_BUF_SIZE,
    utils::{char_arr_to_string, string_to_char_vec},
};
use crate::{
    paragraph_chamber::{Paragraph, Wrap},
    utils::char_vec_to_string,
};

enum InputMode {
    Editing,
    Stopped,
}

struct App {
    input_mode: InputMode,
    received_messages: Vec<String>,
    input_buffer: String,
    cursor_position: usize,
    editor_width: usize,
}
impl Default for App {
    fn default() -> App {
        App {
            input_mode: InputMode::Editing, // 自动进入编辑模式
            received_messages: vec![],
            input_buffer: String::default(),
            cursor_position: 0,
            editor_width: 0,
        }
    }
}
impl App {
    /// get `len()` of the string before your cursor
    fn len_of_str_before_cursor(&self) -> usize {
        let string_before_cursor: String =
            char_arr_to_string(&string_to_char_vec(&self.input_buffer)[0..self.cursor_position]);

        string_before_cursor.len()
    }

    /// get unicode width of the string before your cursor
    fn width_of_str_before_cursor(&self) -> usize {
        let string_before_cursor: String =
            char_arr_to_string(&string_to_char_vec(&self.input_buffer)[0..self.cursor_position]);

        string_before_cursor.width()
    }

    /// get actually occupied width of the string before your cursor
    fn width_occupied_by_str_before_cursor(&self) -> usize {
        let char_arr_before_cursor =
            &string_to_char_vec(&self.input_buffer)[0..self.cursor_position];

        let mut width: usize = 0;
        for ch in char_arr_before_cursor {
            let ch_width = ch.width().unwrap_or_default();
            let mut additional_width = (width + ch_width) % self.editor_width;
            if width + ch_width <= self.editor_width || additional_width >= ch_width {
                additional_width = 0;
            }
            width += ch_width + additional_width;
        }

        width
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
                            app.cursor_position = 0;
                        }
                        KeyCode::Char(ch) => {
                            if app.input_buffer.as_bytes().len() < MSG_BUF_SIZE {
                                app.input_buffer.insert(app.len_of_str_before_cursor(), ch);
                                app.cursor_position += 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if app.cursor_position > 0 {
                                if app.cursor_position == app.input_buffer.chars().count() {
                                    app.input_buffer.pop();
                                } else if app.cursor_position > 0 {
                                    let mut chars_in_buffer: Vec<char> =
                                        string_to_char_vec(&app.input_buffer);
                                    chars_in_buffer.remove(app.cursor_position - 1);
                                    app.input_buffer = char_vec_to_string(&chars_in_buffer);
                                }
                                app.cursor_position -= 1;
                            }
                        }
                        KeyCode::Left => {
                            app.cursor_position -= if app.cursor_position > 0 { 1 } else { 0 }
                        }
                        KeyCode::Right => {
                            app.cursor_position +=
                                if app.cursor_position < app.input_buffer.chars().count() {
                                    1
                                } else {
                                    0
                                };
                        }
                        KeyCode::Up => {
                            // move cursor up
                            let str_width_before_cursor = app.width_of_str_before_cursor();
                            let chars_before_cursor =
                                &mut string_to_char_vec(&app.input_buffer)[0..app.cursor_position];
                            let mut width_to_move: usize = 0;
                            let mut steps_to_move: usize = 0;
                            chars_before_cursor.reverse();
                            if app.cursor_position > 0 {
                                for ch in chars_before_cursor {
                                    let ch_width = ch.width().unwrap_or_default();
                                    // calculate width wasted by char which is wider than 1 and should be at end of line
                                    let additional_width = if (str_width_before_cursor
                                        - width_to_move)
                                        % app.editor_width
                                        < ch_width
                                    {
                                        (str_width_before_cursor - width_to_move) % app.editor_width
                                    } else {
                                        0
                                    };
                                    if width_to_move < app.editor_width {
                                        width_to_move +=
                                            ch.width().unwrap_or_default() + additional_width;
                                        steps_to_move += 1;
                                    } else {
                                        break;
                                    }
                                }
                            }
                            app.cursor_position -= steps_to_move;
                        }
                        KeyCode::Down => {
                            // move cursor down
                            let width_occupied = app.width_occupied_by_str_before_cursor();
                            let chars_in_buf = string_to_char_vec(&app.input_buffer);
                            let mut width_to_move: usize = 0;
                            let mut steps_to_move: usize = 0;
                            if app.cursor_position < app.input_buffer.chars().count() {
                                for ch in &chars_in_buf
                                    [app.cursor_position..app.input_buffer.chars().count()]
                                {
                                    let ch_width = ch.width().unwrap_or_default();
                                    // calculate width wasted by char which is wider than 1 and should be at end of line
                                    let additional_width =
                                        if (width_to_move + ch_width + width_occupied)
                                            % app.editor_width
                                            < ch_width
                                        {
                                            (width_to_move + ch_width + width_occupied)
                                                % app.editor_width
                                        } else {
                                            0
                                        };
                                    if width_to_move < app.editor_width {
                                        width_to_move += ch_width + additional_width;
                                        steps_to_move += 1;
                                    } else {
                                        break;
                                    }
                                }
                            }
                            app.cursor_position += steps_to_move;
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
        break_words: false,
    })
    .block(msg_block);
    frame.render_widget(last_msg_received, left_chunks[0]);

    let online_clents = Block::default()
        .borders(Borders::ALL)
        .title("Online clients")
        .title_alignment(Alignment::Left);
    frame.render_widget(online_clents, chunks[1]);

    let editor_title = format!(
        "Press <Enter> to send, cursor position: {}, char num: {}, str len: {}",
        app.cursor_position,
        app.input_buffer.chars().count(),
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
            break_words: true,
        })
        .block(editor_block);
    app.editor_width = left_chunks[1].width as usize - 2;
    let msg_split_width: usize = app.width_occupied_by_str_before_cursor();
    //    char_arr_to_string(&string_to_char_vec(&app.input_buffer)[0..app.cursor_position]).width();

    frame.set_cursor(
        left_chunks[1].x + (msg_split_width % app.editor_width) as u16 + 1,
        left_chunks[1].y + (msg_split_width / app.editor_width) as u16 + 1,
    );
    frame.render_widget(msg_in_editor, left_chunks[1]);
}
