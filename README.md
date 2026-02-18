# Desklist

Windows 桌面备忘/提醒小工具。无边框置顶窗口，最小化到系统托盘，支持定时提醒和周期性事件。

## 功能

- 事件的创建、编辑、删除，支持标题、描述、时间设置
- 四种筛选视图：今天 / 即将 / 已完成 / 全部
- 定时提醒：准时通知 + 可选提前提醒（5/15/30/60 分钟或 1 天）
- 周期性事件：每天、每周、每月重复，可设结束日期
- 完成周期性事件时自动生成下一次
- 系统托盘常驻，关闭/最小化均隐藏到托盘
- 单实例运行

## 技术栈

- **前端**：Vue 3 + TypeScript + Vite
- **桌面框架**：Tauri 2
- **数据库**：SQLite（通过 `tauri-plugin-sql` 前端直接查询）
- **提醒调度**：Rust + Tokio 异步轮询
- **通知**：`tauri-plugin-notification`

## 开发

```bash
# 安装依赖
npm install

# 开发模式（Vite + Rust 热重载）
npm run tauri dev

# 生产构建
npm run tauri build
```

## 许可证

[MIT](LICENSE)
