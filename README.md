# miniRtspServer

![license](https://img.shields.io/badge/license-MIT-orange)
![prs](https://img.shields.io/badge/PRs-welcome-brightgreen)
![poweredby](https://img.shields.io/badge/powered%20by-qufeiyan-red)

`miniRtspServer` 是一个使用 Rust 语言编写的轻量级 RTSP（Real Time Streaming Protocol）服务器。它旨在为实时流媒体应用提供高效、稳定且易于集成的解决方案。

## 特性

- **轻量级设计**：对系统资源的占用极低，能够在资源受限的环境中稳定运行，如嵌入式设备。
- **高性能**：借助 Rust 的高效内存管理和并发模型，能够快速处理大量的 RTSP 请求，确保实时流的低延迟传输。
- **安全性**：Rust 的内存安全特性从根本上避免了许多常见的安全漏洞，如缓冲区溢出和悬空指针问题。
- **易于集成**：提供了简洁的 API 接口，方便开发者将其集成到自己的项目中。

## 安装

### 前提条件
确保你已经安装了 Rust 开发环境。如果尚未安装，可以从 [Rust 官方网站](https://www.rust-lang.org/tools/install) 下载并安装。

### 克隆仓库
```bash
git clone https://github.com/qufeiyan/miniRtspServer.git
cd miniRtspServer
```

### 构建项目
```bash
cargo build --release
```

## 使用方法

### 启动服务器
构建完成后，你可以使用以下命令启动 `miniRtspServer`：
```bash
./target/release/miniRtspServer
```

### ~~自定义配置~~
~~你可以通过修改配置文件或命令行参数来定制服务器的行为。例如，指定监听端口、日志级别等。~~

### 集成到其他项目
如果你想将 `miniRtspServer` 集成到自己的 Rust 项目中，可以在 `Cargo.toml` 文件中添加以下依赖：
```toml
[dependencies]
miniRtspServer = { git = "https://github.com/qufeiyan/miniRtspServer.git" }
```

然后在代码中引入并使用：
```rust
extern crate miniRtspServer;

fn main() {
    // 初始化并启动服务器
    miniRtspServer::start_server();
}
```

## 贡献

我们欢迎社区的贡献！如果你发现了 bug、有新的功能建议或者想要改进文档，请遵循以下步骤：

1. **Fork 仓库**：点击项目页面上的 `Fork` 按钮，将项目复制到你的 GitHub 账户下。
2. **创建新分支**：在你的本地仓库中创建一个新的分支来进行修改。
```bash
git checkout -b my-feature-branch
```
3. **提交修改**：在完成修改后，提交到你的分支。
```bash
git add .
git commit -m "Add new feature: ..."
```
4. **推送分支**：将你的分支推送到你的 GitHub 仓库。
```bash
git push origin my-feature-branch
```
5. **创建 Pull Request**：在 GitHub 上创建一个 Pull Request，描述你的修改内容和目的。

## 许可证

本项目采用 [MIT 许可证](LICENSE) 进行许可。

## 联系我们

如果你有任何问题或建议，可以通过以下方式联系我们：

- **GitHub Issues**：在项目的 [Issues 页面](https://github.com/qufeiyan/miniRtspServer/issues) 提交问题。
- **Email**：[2491411913@qq.com](mailto:2491411913@qq.com)

感谢你的关注和支持！