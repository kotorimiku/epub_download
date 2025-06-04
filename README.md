# epub_download

一个用于下载网络小说并生成 EPUB 电子书的桌面应用，支持多站点切换、章节内容解密、封面与目录自动生成。

## 功能特性

- 支持多站点（如 bilinovel、linovelib）小说下载
- 自动生成 EPUB 格式电子书，包含封面、目录、章节内容与插图
- 支持章节内容解密与图片下载
- 可自定义输出目录、章节命名模板
- 简洁易用的桌面界面

## 安装与构建

### 前置条件

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/en)
- [pnpm](https://pnpm.io/installation)

### 步骤

1. 克隆本仓库

   ```sh
   git clone https://github.com/kotorimiku/epub_download
   cd epub_download
   ```

2. 安装依赖

   ```sh
   pnpm install
   ```

3. 构建项目

   ```sh
   pnpm tauri build
   ```

4. 运行开发环境

   ```sh
   pnpm tauri dev
   ```

## 使用说明

1. 启动应用后，填写目标小说的书籍 ID 或链接，选择站点与输出目录。
2. 可在“配置”界面自定义章节命名模板、请求间隔、Cookie 等参数。
3. 点击“下载”按钮，等待进度完成后，即可在指定目录获得 EPUB 文件。

## 章节命名模板说明

- `{{book_title}}`：书名
- `{{chapter_title}}`：章节名
- `{{chapter_number}}`：章节编号
- `{{chapter_number:x}}`：章节编号，前补零至 x 位

示例：

- `0`：`{{book_title}}-{{chapter_title}}`
- `1`：`{{book_title}}-[{{chapter_number}}]{{chapter_title}}`
- `2`：`[{{chapter_number}}]{{chapter_title}}`
- `3`：`[{{chapter_number:2}}]{{chapter_title}}`

## 致谢

- [Tauri](https://tauri.app/)
- [Vue.js](https://vuejs.org/)
- [Naive UI](https://www.naiveui.com/)

---
