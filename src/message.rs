use std::fmt::Display;

pub enum MessageType {
    ClientLogIn,
    ClientExit,
    ClientListUpdate,
    TextMessage,
    Error,
}
impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::ClientLogIn => write!(f, "ClientLogin"),
            MessageType::ClientExit => write!(f, "ClientExit"),
            MessageType::ClientListUpdate => write!(f, "ClientListUpdate"),
            MessageType::TextMessage => write!(f, "TextMessage"),
            MessageType::Error => write!(f, "Error"),
        }
    }
}
impl MessageType {
    /// Convert string to `MessageType`
    /// ```rust
    /// "ClientLogin"
    /// ```
    /// --->
    /// ```rust
    /// MessageType::ClientLogIn
    /// ```
    pub fn convert_to_msg_type(msg_type_str: &str) -> MessageType {
        match msg_type_str {
            "ClientLogin" => MessageType::ClientLogIn,
            "ClientExit" => MessageType::ClientExit,
            "ClientListUpdate" => MessageType::ClientListUpdate,
            "TextMessage" => MessageType::TextMessage,
            _ => MessageType::Error,
        }
    }
}

pub struct Message {
    pub msg_type: MessageType,
    pub msg_sender: String,
    pub msg_content: String,
}
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg_str = [
            self.msg_type.to_string(),
            self.msg_sender.clone(),
            self.msg_content.clone(),
        ]
        .join(",");
        write!(f, "{}", msg_str)
    }
}
impl Message {
    /// Convert a formatted string to `Message`
    /// ```rust
    /// "AMsgType,name_or_id,xxxxx"
    /// ```
    /// --->
    /// ```rust
    /// Message {
    ///     msg_type: MessageType::AMsgType,
    ///     sender_name: name_or_id,
    ///     msg_content: xxxxx,
    /// }
    /// ```
    pub fn convert_to_msg(msg_str: &str) -> Message {
        let msg_info: Vec<&str> = msg_str.split(',').collect();
        if msg_info.len() < 3 {
            Message {
                msg_type: MessageType::Error,
                msg_sender: msg_info[1].to_string(),
                msg_content: "Msg format error.".to_string(),
            }
        } else {
            Message {
                msg_type: MessageType::convert_to_msg_type(msg_info[0]),
                msg_sender: msg_info[1].to_string(),
                msg_content: msg_info[2..].join(","),
            }
        }
    }

    /// Convert a `Message` to a `String` that just contains
    /// basic infomation of the message
    pub fn to_brief_string(&self) -> String {
        format!("{}: {}", self.msg_sender, self.msg_content)
    }
}
