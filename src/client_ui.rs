use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
    str::from_utf8,
    sync::mpsc,
    thread,
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders},
    Frame, Terminal,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    consts::MSG_BUF_SIZE,
    message::{Message, MessageType},
    utils::{char_arr_to_string, string_to_char_vec},
};
use crate::{
    paragraph_chamber::{Paragraph, Wrap},
    utils::char_vec_to_string,
};

enum AppFocus {
    Editor,
    MsgList,
    ClientList,
}

struct App {
    focus: AppFocus,
    received_messages: Vec<Message>,
    input_buffer: String,
    cursor_position: usize,
    editor_width: usize,
    stream: Option<TcpStream>,
}
impl Default for App {
    fn default() -> App {
        App {
            focus: AppFocus::Editor, // default mode
            received_messages: vec![],
            input_buffer: String::default(),
            cursor_position: 0,
            editor_width: 0,
            stream: None,
        }
    }
}
impl App {
    /// Get `len()` of the string before your cursor.
    ///
    /// For example, string `"啊a"` is before your cursor,
    /// the function will return `4`.
    fn len_of_str_before_cursor(&self) -> usize {
        let string_before_cursor: String =
            char_arr_to_string(&string_to_char_vec(&self.input_buffer)[0..self.cursor_position]);

        string_before_cursor.len()
    }

    /// Get unicode width of the string before your cursor.
    ///
    /// For example, string `"啊a"` is before your cursor,
    /// the function will return `3`.
    fn width_of_str_before_cursor(&self) -> usize {
        let string_before_cursor: String =
            char_arr_to_string(&string_to_char_vec(&self.input_buffer)[0..self.cursor_position]);

        string_before_cursor.width()
    }

    /// Get actually occupied width of the string before your cursor.
    ///
    /// For example, in such a block, width for string is `10`:
    /// ```shell
    /// ┌──────────┐
    /// │123456789 │
    /// │十        │
    /// └──────────┘
    /// ```
    /// The width of string `"123456789十"` shoud be `11`,
    /// but actually occupied width will be `12`.
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

    /// Get number of actually occupied lines by the msgs received.
    fn lines_occupied_by_msg_received(&self) -> usize {
        let mut lines: usize = 0;
        for msg in &self.received_messages {
            let msg_str = msg.to_brief_string();
            let mut occupied_width = 0;
            for ch in msg_str.chars() {
                let ch_width = ch.width().unwrap_or_default();
                let additional_width = (occupied_width + ch_width) % self.editor_width;
                if occupied_width + ch_width < self.editor_width {
                    occupied_width += ch_width;
                } else {
                    occupied_width = additional_width;
                    lines += 1;
                }
            }
            if occupied_width > 0 {
                lines += 1;
            }
        }
        lines
    }

    /// remove a char just before the cursor
    fn remove_a_char_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            if self.cursor_position == self.input_buffer.chars().count() {
                self.input_buffer.pop();
            } else if self.cursor_position > 0 {
                let mut chars_in_buffer: Vec<char> = string_to_char_vec(&self.input_buffer);
                chars_in_buffer.remove(self.cursor_position - 1);
                self.input_buffer = char_vec_to_string(&chars_in_buffer);
            }
            self.cursor_position -= 1;
        }
    }

    /// move cursor to the line above
    fn move_cursor_up(&mut self) {
        let str_width_before_cursor = self.width_of_str_before_cursor();
        let chars_before_cursor =
            &mut string_to_char_vec(&self.input_buffer)[0..self.cursor_position];
        let mut width_to_move: usize = 0;
        let mut steps_to_move: usize = 0;
        chars_before_cursor.reverse();
        if self.cursor_position > 0 {
            for ch in chars_before_cursor {
                let ch_width = ch.width().unwrap_or_default();
                // calculate width wasted by char which is wider than 1 and should be at end of line
                let additional_width =
                    if (str_width_before_cursor - width_to_move) % self.editor_width < ch_width {
                        (str_width_before_cursor - width_to_move) % self.editor_width
                    } else {
                        0
                    };
                if width_to_move < self.editor_width {
                    width_to_move += ch_width + additional_width;
                    steps_to_move += 1;
                } else {
                    break;
                }
            }
        }
        self.cursor_position -= steps_to_move;
    }

    /// move cursor to the line below
    fn move_cursor_down(&mut self) {
        let width_occupied = self.width_occupied_by_str_before_cursor();
        let chars_in_buf = string_to_char_vec(&self.input_buffer);
        let mut width_to_move: usize = 0;
        let mut steps_to_move: usize = 0;
        if self.cursor_position < self.input_buffer.chars().count() {
            for ch in &chars_in_buf[self.cursor_position..self.input_buffer.chars().count()] {
                let ch_width = ch.width().unwrap_or_default();
                // calculate width wasted by char which is wider than 1 and should be at end of line
                let additional_width =
                    if (width_to_move + ch_width + width_occupied) % self.editor_width < ch_width {
                        (width_to_move + ch_width + width_occupied) % self.editor_width
                    } else {
                        0
                    };
                if width_to_move < self.editor_width {
                    width_to_move += ch_width + additional_width;
                    steps_to_move += 1;
                } else {
                    break;
                }
            }
        }
        self.cursor_position += steps_to_move;
    }

    /// send msg in `input_buffer` to server
    fn send_msg(&mut self) -> std::io::Result<()> {
        // do not send empty or blank string
        if self.input_buffer.is_empty() || self.input_buffer.trim().is_empty() {
            return Ok(());
        }
        let msg_content: String = self.input_buffer.drain(..).collect();
        let msg = Message {
            msg_type: MessageType::TextMessage,
            msg_sender: self
                .stream
                .as_ref()
                .unwrap()
                .local_addr()
                .expect("Failed to get local addr.")
                .to_string(),
            msg_content,
        };
        self.stream
            .as_ref()
            .unwrap()
            .write(msg.to_string().as_bytes())?;
        self.cursor_position = 0;

        Ok(())
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // connect to server
    let stream = TcpStream::connect("127.0.0.1:9999")?;
    let mut stream_clone = stream.try_clone()?;
    app.stream = Some(stream);

    let (msg_sender, msg_receiver) = mpsc::channel::<Message>();

    // create a thread to read msg from server
    thread::spawn(move || {
        let mut buffer: Vec<u8> = vec![0; MSG_BUF_SIZE];
        loop {
            if let Ok(msg_size) = stream_clone.read(&mut buffer) {
                if msg_size > 0 {
                    let msg = Message::convert_to_msg(from_utf8(&buffer[..msg_size]).unwrap());
                    msg_sender
                        .send(msg)
                        .expect("Failed to send msg to msg_receiver.");
                }
            } else {
                // should try to re-connect, or just quit
                let msg = Message {
                    msg_type: MessageType::Error,
                    msg_sender: "localhost".to_string(),
                    msg_content: "Lost connection.".to_string(),
                };
                msg_sender
                    .send(msg)
                    .expect("Failed to send msg to msg_receiver.");
                break;
            }
        }
    });

    // draw ui in this loop
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        // handle received msg
        if let Ok(msg) = msg_receiver.try_recv() {
            match msg.msg_type {
                MessageType::ClientListUpdate => {}
                MessageType::TextMessage => {
                    app.received_messages.push(msg);
                }
                MessageType::Error => {}
                _ => {}
            }
        }

        // check events 10 times every second
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.focus {
                    AppFocus::Editor => {
                        match key.code {
                            KeyCode::Enter => {
                                app.send_msg().expect("Failed to send msg with app.");
                            }
                            KeyCode::Char(ch) => {
                                // length of msg should be shorter
                                // for there are msg type and sender in Message
                                if app.input_buffer.as_bytes().len() < MSG_BUF_SIZE {
                                    app.input_buffer.insert(app.len_of_str_before_cursor(), ch);
                                    app.cursor_position += 1;
                                }
                            }
                            KeyCode::Backspace => {
                                app.remove_a_char_before_cursor();
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
                                app.move_cursor_up();
                            }
                            KeyCode::Down => {
                                app.move_cursor_down();
                            }
                            KeyCode::Esc => {
                                // should set input_mode to STOPPED
                                // app.input_mode = InputMode::Stopped;
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    AppFocus::MsgList => match key.code {
                        KeyCode::Up => {}
                        KeyCode::Down => {}
                        _ => {}
                    },
                    AppFocus::ClientList => match key.code {
                        KeyCode::Up => {}
                        KeyCode::Down => {}
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let size = frame.size();

    // split window into two parts
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(size);

    // left part of window includes msg_block ans editor
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chunks[0]);

    // display all msgs received
    // should do some scroll operation to ensure the newest msg appear at bottom of msg_block
    let msg_block = Block::default()
        .borders(Borders::ALL)
        .title("Chamber Message Window")
        .title_alignment(Alignment::Left);
    let msgs_spans: Vec<Spans> = app
        .received_messages
        .iter()
        .map(|i| Spans::from(i.to_brief_string()))
        .collect();
    // scroll to display the newest msg
    let offset_y = if app.lines_occupied_by_msg_received() as u16 <= left_chunks[0].height - 2 {
        0
    } else {
        app.lines_occupied_by_msg_received() as u16 - (left_chunks[0].height - 2)
    };
    let msg_para = Paragraph::new(msgs_spans)
        .wrap(Wrap {
            trim: false,
            break_words: false,
        })
        .block(msg_block)
        .scroll((offset_y, 0));
    frame.render_widget(msg_para, left_chunks[0]);

    // should display online clients
    let online_clents = Block::default()
        .borders(Borders::ALL)
        .title("Online clients")
        .title_alignment(Alignment::Left);
    frame.render_widget(online_clents, chunks[1]);

    // editor is a block to input msgs
    let editor_title = format!(
        "Press <Enter> to send, cursor position: {}, char num: {}, bytes: {}",
        app.cursor_position,
        app.input_buffer.chars().count(),
        app.input_buffer.len()
    );
    let editor_block = Block::default()
        .borders(Borders::ALL)
        .title(editor_title)
        .title_alignment(Alignment::Left)
        .style(match app.focus {
            AppFocus::Editor => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    let msg_in_editor = Paragraph::new(app.input_buffer.as_ref())
        .wrap(Wrap {
            trim: false,
            break_words: true,
        })
        .block(editor_block);
    // update width of editor block
    app.editor_width = left_chunks[1].width as usize - 2;
    // get actually occupied width by msg in editor
    let msg_split_width: usize = app.width_occupied_by_str_before_cursor();
    frame.set_cursor(
        left_chunks[1].x + (msg_split_width % app.editor_width) as u16 + 1,
        left_chunks[1].y + (msg_split_width / app.editor_width) as u16 + 1,
    );
    frame.render_widget(msg_in_editor, left_chunks[1]);
}
