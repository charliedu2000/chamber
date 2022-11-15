# Chamber

*名字叫密室，但并不是真的“密室”，只是个聊天室。*

*You can call it Chamber, but it's not a real CHAMBER, just a chatroom.*

## 目标

*施工中*

- [x] 基于 TCP 连接的服务器和客户端，实现最简单的即时聊天，多个客户端连接服务器；
- [x] 给客户端加上一个~~炫酷的~~ TUI；
  - [x] 实现基本上可用的输入框；
  - [x] 显示消息列表；
  - [ ] 显示当前在线客户端列表（需要完成自定义消息格式）；
  - [ ] 优化上述项目，例如要能够滚动浏览，解决溢出后无法浏览新内容的问题等；
- [ ] 自定义消息格式来包含更多信息，区分被广播的信息的发送方；
- [ ] 使用文件自定义配置，实现客户端自定义昵称等功能；
  - [ ] 更新在线客户端列表时如何避免受到缓冲大小的限制？
- [ ] 完善启动时的命令行参数功能；
- [ ] 命令模式/快捷键菜单；
- [ ] 或许…… Chamber Ver.Web？

## 运行

需要 [Rust 开发环境](https://www.rust-lang.org/learn/get-started)。

```sh
# local server
cargo run -- server
# local client
cargo run -- client
# local client with TUI
cargo run -- ui
```

## 相关项目

[tui-rs](https://github.com/fdehau/tui-rs)

[Phoenix-Chen / tui-rs](https://github.com/Phoenix-Chen/tui-rs/tree/optional_trim_end)
