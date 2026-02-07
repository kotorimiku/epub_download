# EPUB Download

<div align="center">

一个基于 Tauri 的轻量级桌面应用，用于从 bilinovel.com 下载网络小说并生成 EPUB 电子书。

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/tauri-2.0-blue.svg)](https://tauri.app)
[![Vue](https://img.shields.io/badge/vue-3.5-green.svg)](https://vuejs.org)

</div>

## ✨ 特性

- 🚀 **跨平台支持** - 基于 Tauri 构建，支持 Windows、macOS 和 Linux
- 📚 **智能下载** - 自动获取小说章节、封面和元数据
- 🔓 **内容解密** - 自动处理 bilinovel.com 的内容加密和段落乱序
- 📖 **EPUB 生成** - 生成符合标准的 EPUB 3.0 格式电子书
- 🎨 **自动目录** - 自动生成书籍目录和章节导航

## 📦 安装

### 从发行版安装

前往 [Releases](https://github.com/kotorimiku/epub_download/releases) 页面下载最新版本：

- Windows: `epub_download_*.exe`
- macOS: `epub_download_*.dmg`
- Linux: `epub_download_*.AppImage` 或 `epub_download_*_amd64.deb`

### 从源码构建

#### 前置要求

- [Node.js](https://nodejs.org/) 18+
- [Bun](https://bun.sh/) (推荐) 或 npm/pnpm/yarn
- [Rust](https://www.rust-lang.org/) 1.70+
- Tauri 依赖项（参见 [Tauri 前置要求](https://tauri.app/v1/guides/getting-started/prerequisites)）

#### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/kotorimiku/epub_download.git
cd epub_download

# 安装依赖
bun install

# 开发模式运行
bun tauri dev

# 构建生产版本
bun tauri build
```

## ⚙️ 配置

首次运行时会在应用目录生成 `config.json` 配置文件：

```json
{
  "base_url": "https://www.bilinovel.com",
  "output": "./books",
  "template": "{{book_title}}/{{volume_title}}/{{chapter_number:3}}_{{chapter_title}}",
  "cookie": "",
  "user_agent": "Mozilla/5.0 ...",
  "add_catalog": true,
  "retry_times": 3,
  "timeout": 30
}
```

### 配置项说明

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `base_url` | bilinovel 站点地址 | `https://www.bilinovel.com` |
| `output` | EPUB 输出目录 | `./books` |
| `template` | 章节文件命名模板 | `{{book_title}}/{{volume_title}}/...` |
| `cookie` | 登录 Cookie（可选） | `""` |
| `user_agent` | HTTP 请求 User-Agent | 默认浏览器标识 |
| `add_catalog` | 是否生成目录页 | `true` |
| `retry_times` | 下载失败重试次数 | `3` |
| `timeout` | 请求超时时间（秒） | `30` |

### 章节命名模板变量

- `{{book_title}}` - 书籍标题
- `{{volume_title}}` - 卷标题
- `{{chapter_title}}` - 章节标题
- `{{chapter_number}}` - 章节编号
- `{{chapter_number:x}}` - 补零到 x 位的章节编号

## 🔧 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/command.rs` 中定义命令：

```rust
#[tauri::command]
#[specta::specta]
pub async fn my_command(param: String) -> Result<String, CommandError> {
    // 实现逻辑
    Ok(result)
}
```

2. 在 `src-tauri/src/lib.rs` 中注册：

```rust
collect_commands![
    // ...现有命令
    my_command,
]
```

3. 运行 `bun tauri dev` 自动生成 TypeScript 绑定

4. 在 Vue 组件中使用：

```typescript
import { commands } from './bindings'
const result = await commands.myCommand('parameter')
```

### 项目脚本

```bash
# 开发模式
bun tauri dev

# 构建生产版本
bun tauri build

# Rust 代码检查
bun check

# Rust 代码规范检查
bun clippy

# 仅启动前端开发服务器
bun dev
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- Rust: 遵循 `rustfmt` 和 `clippy` 建议
- TypeScript/Vue: 遵循 ESLint 配置
- 提交信息: 使用清晰的描述

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## ⚠️ 免责声明

本工具仅供学习和个人使用，请尊重原作者版权。下载的内容仅供个人阅读，请勿用于商业用途。如果您喜欢某部作品，请支持正版。

## 🔗 相关链接

- [Tauri 官方文档](https://tauri.app/)
- [Vue 3 官方文档](https://vuejs.org/)
- [Rust 官方网站](https://www.rust-lang.org/)

## 💬 联系方式

如有问题或建议，欢迎通过 [Issues](https://github.com/kotorimiku/epub_download/issues) 反馈。

---

<div align="center">
Made with ❤️ by kotorimiku
</div>
