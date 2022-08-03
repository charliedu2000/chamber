# Chamber

*名字叫密室，但并不是真的“密室”，只是个聊天室。*

*You can call it Chamber, but it's not a real CHAMBER, just a chatroom.*

## 目标

*施工中*

- [x] 基于 TCP 连接的服务器和客户端，实现最简单的即时聊天，多个客户端连接服务器；
- [ ] 给客户端加上一个~~炫酷的~~ TUI；
- [ ] 使用文件自定义配置，实现客户端自定义昵称等功能；
- [ ] 自定义消息格式来包含更多信息，区分被广播的信息的发送方；
- [ ] 完善启动时的命令行参数功能；
- [ ] 命令模式或快捷键菜单，命令/菜单功能待定；

## 运行

*自己的 Flag 刚立起来，还没打算提供二进制包。*

需要 [Rust 开发环境](https://www.rust-lang.org/learn/get-started)。

```sh
# local server
cargo run -- server
# local client
cargo run -- client
```

