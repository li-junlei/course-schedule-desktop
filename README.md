# 课程表桌面应用 (Course Schedule Desktop)

<div align="center">

一款基于 **Tauri 2.0 + Vue 3 + TypeScript** 构建的现代化跨平台课程表管理桌面应用

[![Version](https://img.shields.io/badge/version-1.4.0-blue.svg)](https://github.com/lijunlei/course-schedule-desktop)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D.svg)](https://vuejs.org/)

[功能特性](#功能特性) • [快速开始](#快速开始) • [使用指南](#使用指南) • [技术栈](#技术栈) • [开发文档](#开发文档)

</div>

---

## ✨ 功能特性

### 📚 核心功能
- **多课表管理** - 创建、切换、编辑、删除多个课表，支持拖拽排序
- **智能导入** - 内置浏览器一键导入，剪贴板自动传输
- **自定义时间表** - 支持多套节次时间配置，灵活适配不同学校
- **动态周数** - 周次选择器根据课表设置动态显示（支持 18/20/25 周等）

### 🎨 用户界面
- **周次预览** - 点阵视图展示每周课程分布，一目了然
- **滑动切换** - 手势滑动切换周次，自动适配周数范围
- **课程详情** - 底部抽屉展示完整课程信息
- **深色模式** - 支持手动切换和跟随系统主题

### ⚙️ 外观定制
- **网格辅助线** - 可开关的虚线网格，无上边和左边框
- **卡片透明度** - 可调节课程卡片不透明度（50-100%）
- **信息显示控制** - 自定义显示教师/地点信息
- **自定义背景** - 支持上传本地图片作为课表背景

### 📅 智能提示
- **学期状态** - 自动显示"未开学"/"学期已结束"状态
- **今日高亮** - 强调显示当前日期，快速定位
- **本地时间** - 基于本地时间计算，准确无误

---

## 🚀 快速开始

### 环境要求

- **Node.js** 18+
- **Rust** 1.70+ (含 Cargo)
- **npm** 或 yarn

### 安装步骤

1. **克隆仓库**
   ```bash
   git clone https://github.com/lijunlei/course-schedule-desktop.git
   cd course-schedule-desktop
   ```

2. **安装依赖**
   ```bash
   npm install
   ```

3. **开发模式**
   ```bash
   npm run tauri dev
   ```

4. **构建应用**
   ```bash
   npm run tauri build
   ```

### 获取安装包

从 [Releases](https://github.com/lijunlei/course-schedule-desktop/releases) 页面下载最新版本的安装包：

- **Windows**: `.msi` 或 `.exe` 安装包
- **macOS**: `.dmg` 安装包
- **Linux**: `.AppImage` 或 `.deb` 包

---

## 📖 使用指南

### 导入课表

1. 点击左上角菜单按钮
2. 选择"导入课表"
3. 在打开的浏览器中登录教务系统
4. 导航到课表页面
5. 点击右下角"导入当前课表"浮动按钮
6. 关闭浏览器窗口，应用自动导入

### 管理多个课表

- **创建**: 通过"导入课表"创建新课表
- **切换**: 在"课表管理"中点击课表卡片切换
- **排序**: 拖拽课表卡片调整顺序
- **编辑**: 点击编辑图标修改课表信息和学期设置
- **删除**: 点击删除图标移除不需要的课表

### 自定义时间表

1. 打开"课表管理" → 编辑课表
2. 在"时间表"部分点击"编辑"
3. 修改每节课的开始/结束时间
4. 可创建多个时间表并应用到不同课表
5. 点击"应用到所有课表"统一时间配置

### 修改学期周数

1. 打开"课表管理" → 点击要编辑的课表卡片上的编辑图标
2. 在"学期设置"中修改"结束周次"
3. 保存后，周次选择器会自动显示正确的周数范围
4. 滑动切换周次时也会自动适配新的周数范围

### 外观设置

- **网格辅助线**: 在"设置" → "课表外观设置"中开启
- **卡片透明度**: 拖动滑块调整（50-100%）
- **信息显示**: 开关"显示授课教师"或"显示上课地点"
- **背景图片**: 点击菜单 → "上传背景"选择本地图片

---

## 🛠️ 技术栈

### 前端技术
- **框架**: Vue 3 (Composition API + `<script setup>`)
- **类型系统**: TypeScript 5.6
- **构建工具**: Vite 6.0
- **UI 组件库**: Element Plus 2.13
- **状态管理**: Vue 3 Reactivity API
- **拖拽库**: vuedraggable 4.1.0
- **样式**: SCSS + Scoped CSS

### 后端技术（Rust）
- **框架**: Tauri 2.0
- **HTTP 客户端**: reqwest 0.11
- **HTML 解析**: scraper 0.25
- **加密**: des 0.8, md5 0.7
- **序列化**: serde/serde_json
- **时间处理**: chrono 0.4

### 开发工具
- **包管理器**: npm
- **类型检查**: vue-tsc
- **代码规范**: TypeScript strict mode

---

## 📂 项目结构

```
course-schedule-desktop/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   ├── composables/        # 组合式函数
│   ├── utils/             # 工具函数
│   └── App.vue            # 根组件
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── lib.rs         # Tauri 命令定义
│   │   ├── models.rs      # 数据模型
│   │   ├── parser.rs      # HTML 解析
│   │   ├── storage.rs     # 本地存储
│   │   └── crypto.rs      # 加密模块
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── package.json           # 前端依赖配置
└── README.md              # 项目文档
```

---

## 📸 界面预览

### 主界面
- 周次选择器（点阵视图）
- 课程网格布局
- 日期导航栏
- 课程卡片展示

### 课表管理
- 拖拽排序
- 多课表切换
- 时间表编辑
- 学期设置

### 外观设置
- 网格辅助线
- 卡片透明度
- 信息显示控制
- 背景图片

---

## 🔧 开发指南

### 添加新的教务系统支持

1. 修改 `src-tauri/src/parser.rs` 中的 `parse_course_html` 函数
2. 适配目标教务系统的 HTML 表格结构
3. 处理特殊的时间格式、节次表示、周次格式
4. 测试解析结果的准确性

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/lib.rs` 添加 `#[tauri::command]` 函数
2. 在 `invoke_handler!` 宏中注册命令
3. 前端使用 `invoke()` 调用

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('command_name', { param: value });
```

### 构建配置

```bash
# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 仅构建前端
npm run build
```

---

## 🌐 常见问题

<details>
<summary><b>Q: 为什么需要剪贴板权限？</b></summary>

A: 由于 Tauri 2 的安全限制，外部 HTTPS 网页（教务系统）无法直接访问 Tauri API。因此使用剪贴板作为数据传输通道。点击"导入当前课表"按钮会将 HTML 源代码复制到剪贴板，前端再自动读取。
</details>

<details>
<summary><b>Q: 课表数据存储在哪里？</b></summary>

A:
- **Windows**: `C:\Users\<用户>\AppData\Local\course-schedule\`
- **macOS**: `~/Library/Application Support/course-schedule/`
- **Linux**: `~/.local/share/course-schedule/`
</details>

<details>
<summary><b>Q: 如何支持单双周课程？</b></summary>

A: 解析器会自动识别周次范围，前端会根据：
- `weeks` 数组：包含该课程的周次列表
- `week_type` 字段：1=单周，2=双周，0=全周

来判断是否在当前周显示课程。
</details>

<details>
<summary><b>Q: 如何修改学期周数？</b></summary>

A:
1. 打开"课表管理" → 点击要编辑的课表卡片上的编辑图标
2. 在"学期设置"中修改"结束周次"
3. 保存后，周次选择器会自动显示正确的周数范围
4. 滑动切换周次时也会自动适配新的周数范围
</details>

---

## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 修复 bug
- `docs:` 文档更新
- `style:` 代码格式调整
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建/工具链相关

---

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。

---

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Element Plus](https://element-plus.org/) - Vue 3 UI 组件库
- [Rust](https://www.rust-lang.org/) - 系统编程语言

---

## 📮 联系方式

- **作者**: Li Junlei
- **项目地址**: [https://github.com/lijunlei/course-schedule-desktop](https://github.com/lijunlei/course-schedule-desktop)
- **问题反馈**: [Issues](https://github.com/lijunlei/course-schedule-desktop/issues)

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️ Star 支持一下！**

Made with ❤️ by Li Junlei

</div>
