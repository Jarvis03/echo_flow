# LingoDesk V0.1 开发规格说明

## 1. 项目目标

开发一个 Windows 桌面级 AI 翻译与工作英语学习助手。

核心目标不是针对 Slack 单独开发插件，而是通过系统级能力统一支持：

* Slack Desktop
* WhatsApp Desktop
* 微信
* Microsoft Teams
* Outlook
* Chrome / Edge
* PDF 阅读器
* Jira / Confluence
* 其他 Windows 桌面软件

用户不需要复制内容到 Google Translate 或 ChatGPT。

LingoDesk 应实现：

**选中文字 → 快捷键 → 显示翻译**

以及：

**输入中文 → 快捷键 → 翻译成英文 → 写回当前输入框**

同时保存用户主动收藏的内容，形成个人工作英语学习库。

---

# 2. 产品原则

开发时始终遵循以下原则。

## 2.1 App 无关

核心功能不依赖 Slack API。

LingoDesk 不应该判断：

“当前是不是 Slack？”

而应该判断：

“当前有没有选中的文字？”

以及：

“当前有没有可编辑输入框？”

这样才能支持不同软件。

---

## 2.2 系统级统一交互

所有软件统一使用同一套快捷键。

默认快捷键：

```text
Alt + Q
翻译当前选中文字

Alt + Enter
将当前输入内容从中文翻译成英文

Alt + Shift + Q
屏幕区域截图 OCR + 翻译

Alt + S
收藏当前翻译内容
```

快捷键后续必须可以自定义。

---

## 2.3 不自动发送

LingoDesk 可以：

* 修改输入框
* 替换文字
* 复制翻译结果

但绝对不自动点击 Send。

最终发送必须由用户完成。

---

# 3. 第一阶段平台

V0.1：

只支持 Windows。

暂不开发：

* macOS
* Linux
* Android
* iOS

但代码结构应允许后续增加 macOS Accessibility API。

---

# 4. 推荐技术栈

桌面框架：

```text
Tauri
```

前端：

```text
React
TypeScript
```

后端：

```text
Rust
```

本地数据库：

```text
SQLite
```

状态管理可以选择：

```text
Zustand
```

UI 可以选择：

```text
Tailwind CSS
```

不要引入过重的 UI Framework。

---

# 5. 总体架构

建议架构：

```text
Windows
│
├── Global Hotkey Manager
│
├── Text Capture Engine
│     ├── UI Automation
│     ├── Clipboard fallback
│     └── OCR fallback
│
├── Text Injection Engine
│     ├── UI Automation
│     └── Clipboard paste fallback
│
├── Translation Engine
│
├── Floating Window
│
├── Learning Database
│
└── Settings
```

核心设计原则：

```text
Capture
↓
Process
↓
Display
↓
Inject
↓
Store
```

不同 App 不需要单独写业务逻辑。

---

# 6. Text Capture Engine

这是整个项目最关键模块之一。

目标：

从当前正在使用的软件中获取文字。

需要实现三层机制。

---

# 7. 第一层：Windows UI Automation

优先尝试 Windows UI Automation API。

目标：

获取：

* 当前 Focus Element
* Selected Text
* Editable Text
* Control Type
* Process Name
* Window Title

例如：

Slack 输入框。

或者：

Chrome 中被选中的文字。

设计接口：

```typescript
interface TextCaptureResult {
  success: boolean
  text?: string
  source?: "uia" | "clipboard" | "ocr"
  appName?: string
  windowTitle?: string
}
```

Rust 层实现：

```text
Windows UI Automation
```

需要封装为：

```text
capture_selected_text()
```

和：

```text
capture_current_input()
```

---

# 8. 第二层：Clipboard Fallback

UI Automation 失败时：

自动使用 Clipboard。

逻辑：

```text
保存当前 clipboard
↓
模拟 Ctrl+C
↓
等待 50~150ms
↓
读取 clipboard
↓
恢复原 clipboard
```

注意：

一定要恢复原用户 Clipboard。

不能破坏用户之前复制的内容。

函数：

```text
capture_via_clipboard()
```

Clipboard 是第一版兼容性最重要的兜底方式。

---

# 9. 第三层：OCR

当：

```text
UI Automation
+
Clipboard
```

均无法获取文字时：

使用截图 OCR。

V0.1 OCR 可以作为独立功能：

```text
Alt + Shift + Q
```

用户框选屏幕区域。

流程：

```text
Screen Capture
↓
OCR
↓
Text
↓
Translation
↓
Floating Window
```

OCR 推荐优先寻找：

```text
Windows OCR API
```

或者：

```text
PaddleOCR
```

不要在第一版做过于复杂的图像处理。

---

# 10. 文本写回 Text Injection

第二个核心模块。

用户例如在 Slack 输入：

```text
这个问题我们还在检查，有进展我通知你
```

按：

```text
Alt + Enter
```

LingoDesk：

```text
读取当前输入框
↓
AI 翻译
↓
得到英文
↓
替换当前输入框
```

得到：

```text
We're still checking this issue. I'll keep you posted if there's any update.
```

但不发送。

---

# 11. Text Injection 优先级

第一种：

Windows UI Automation：

```text
ValuePattern
TextPattern
```

如果当前 Control 支持修改文本：

直接替换。

第二种：

Clipboard + Keyboard Simulation。

逻辑：

```text
复制翻译结果
↓
Ctrl+A
↓
Ctrl+V
```

注意：

必须保证目标焦点仍然是原来的输入框。

---

# 12. 翻译功能

Translation Engine 先实现三种模式。

## READ

英文 → 中文。

用于理解别人说什么。

Prompt 目标：

* 保留技术术语
* 准确
* 中文自然
* 不进行额外扩展

---

## REPLY

中文 → 英文。

目标：

生成：

* 自然
* 简单
* 工作场景可用
* 不过度正式
* 不使用过于高级词汇

Prompt 示例：

```text
Translate the following Chinese message into natural workplace English.

Requirements:

1. Keep the English simple and easy to learn.
2. Prefer common vocabulary.
3. Prefer short sentences.
4. Use a natural Slack/workplace tone.
5. Do not make the message more formal than necessary.
6. Preserve technical terms accurately.
7. Do not add information that is not in the original message.
8. Return only the translated English.

Chinese:
{{TEXT}}
```

---

## EXPLAIN

用于学习。

输出：

```json
{
  "translation": "",
  "keywords": [],
  "expressions": []
}
```

例如：

输入：

```text
I'll keep you posted once we have an update.
```

输出：

```json
{
  "translation": "有进展后我会及时通知你。",
  "keywords": [
    {
      "text": "update",
      "meaning": "更新 / 进展"
    }
  ],
  "expressions": [
    {
      "text": "keep you posted",
      "meaning": "及时告诉你最新进展"
    }
  ]
}
```

---

# 13. AI Provider 架构

不要把 OpenAI API 写死。

定义：

```typescript
interface AIProvider {
  translateRead(text: string): Promise<string>
  translateReply(text: string): Promise<string>
  explain(text: string): Promise<ExplainResult>
}
```

Provider：

```text
OpenAIProvider
```

后续允许：

```text
GeminiProvider
ClaudeProvider
LocalLLMProvider
```

V0.1 只需要实现 OpenAIProvider。

API Key：

存储在：

```text
Windows Credential Manager
```

不要明文存在 SQLite。

---

# 14. Floating Translation Window

按：

```text
Alt + Q
```

后显示一个非常轻量的浮窗。

例如：

```text
┌──────────────────────────────────┐
│ LingoDesk                     × │
│                                  │
│ Could you verify the GPS data?   │
│                                  │
│ 你能确认一下 GPS 数据吗？        │
│                                  │
│ verify                           │
│ 确认 / 验证                      │
│                                  │
│ ☆ Save     Copy     Explain      │
└──────────────────────────────────┘
```

要求：

* Always on top
* 无任务栏图标
* 轻量
* 自动定位在鼠标附近
* ESC 关闭
* 点击其他区域可关闭
* 支持复制翻译

浮窗不能打断用户当前工作。

---

# 15. 翻译状态

浮窗至少需要：

```text
Loading
Success
Error
```

Loading：

```text
Translating...
```

Error：

```text
Translation failed
Retry
```

不要让 App 卡住。

---

# 16. Learning Database

SQLite 数据结构第一版可以非常简单。

表：

```sql
translations
```

字段：

```text
id
created_at
source_app
source_window

original_text
translated_text

direction

saved
```

direction：

```text
EN_TO_ZH
ZH_TO_EN
```

---

# 17. 收藏系统

默认：

翻译内容可以进入短期历史。

只有用户点击：

```text
☆
```

变成：

```text
★
```

才进入长期 Learning 数据。

---

# 18. Vocabulary

第二张表：

```sql
vocabulary
```

结构：

```text
id
word
meaning
example
created_at
frequency
favorite
```

---

# 19. Expression

第三张表：

```sql
expressions
```

结构：

```text
id
expression
meaning
example
created_at
frequency
favorite
```

例如：

```text
keep you posted
```

---

# 20. Learning 页面

V0.1 做简单版本。

主界面：

```text
Today
```

展示：

```text
Translations       18
Replies             6
Saved               3
```

下面：

```text
Today's Saved Expressions
```

例如：

```text
keep you posted

有进展及时通知你

I'll keep you posted once we have an update.
```

---

# 21. History 页面

展示翻译历史。

结构：

```text
Today
────────────────────

Slack

Could you verify this issue?

你能确认一下这个问题吗？
```

允许：

```text
Search
Delete
Favorite
Copy
```

---

# 22. Settings

V0.1 Settings：

## AI

```text
Provider
API Key
Model
```

## Translation

默认 Reply Mode：

```text
Simple Workplace English
```

## Hotkeys

```text
Translate Selection
Translate Reply
OCR Translation
Save
```

## Privacy

提供：

```text
Save History ON/OFF
```

以及：

```text
Auto Delete History
```

可选：

```text
7 Days
30 Days
Never
```

---

# 23. 系统托盘

LingoDesk 启动后常驻 Tray。

右键：

```text
Open LingoDesk
Translation History
Settings
Pause Hotkeys
Quit
```

---

# 24. App 启动

支持：

```text
Launch at Windows startup
```

但默认关闭。

---

# 25. 日志

开发版本需要日志。

例如：

```text
capture.uia.success
capture.uia.fail
capture.clipboard.success
translation.start
translation.success
inject.success
```

不要记录完整的用户消息内容到 Debug log。

最多记录：

```text
text_length
```

避免泄露隐私。

---

# 26. Privacy

因为用户可能处理公司 Slack 和邮件：

必须遵守：

```text
Local First
```

本地存：

* History
* Vocabulary
* Expressions
* Settings

只有用户需要翻译的文本发送给 AI Provider。

---

# 27. V0.1 不做的功能

第一版不要实现：

* Slack API
* WhatsApp API
* 微信 API
* 自动读取整个聊天窗口
* 自动回复
* 自动发送
* 联系人识别
* 消息监听
* 后台监控聊天内容
* 自动总结所有聊天
* 多设备同步
* 账号系统
* 云端数据库
* 团队协作
* 浏览器插件
* macOS

这些全部放到后续版本。

---

# 28. MVP 开发顺序

严格按照以下顺序开发。

## Milestone 1

完成：

```text
Global Hotkey
+
Clipboard Capture
+
Floating Window
```

验收：

在：

```text
Notepad
Chrome
Slack
```

选中文字。

按：

```text
Alt + Q
```

程序能获得文字并弹窗。

此阶段翻译可以先 Mock。

---

# 29. Milestone 2

接入 AI Translation。

实现：

```text
English → Chinese
```

验收：

Slack 中：

选中英文消息

↓

Alt + Q

↓

显示中文。

---

# 30. Milestone 3

实现：

```text
Chinese → English
```

用户：

Slack 输入中文

↓

Alt + Enter

↓

自动替换成英文。

必须：

```text
不发送
```

---

# 31. Milestone 4

加入：

```text
SQLite
History
Favorite
```

实现：

```text
★ Save
```

---

# 32. Milestone 5

加入：

```text
UI Automation
```

Capture 策略变成：

```text
UI Automation
↓
失败
↓
Clipboard
```

---

# 33. Milestone 6

实现：

```text
Screenshot
+
OCR
```

快捷键：

```text
Alt + Shift + Q
```

---

# 34. 兼容性测试

至少测试：

```text
Slack Desktop
Chrome
Edge
Notepad
VS Code
Microsoft Word
Outlook
WeChat
WhatsApp Desktop
Teams
```

记录每个 App：

```text
UI Automation Capture
Clipboard Capture
UI Automation Injection
Clipboard Injection
```

结果。

例如：

| App      | Capture       | Injection |
| -------- | ------------- | --------- |
| Slack    | Clipboard     | Clipboard |
| Chrome   | UIA/Clipboard | Clipboard |
| WeChat   | Clipboard     | Clipboard |
| Outlook  | UIA           | UIA       |
| WhatsApp | Clipboard     | Clipboard |

不要为了某个 App 写大量硬编码。

---

# 35. Adapter 机制

如果未来某些 App 确实需要特殊处理：

设计：

```text
AppAdapter
```

接口：

```typescript
interface AppAdapter {
  detect(): boolean
  capture(): string | null
  inject(text: string): boolean
}
```

默认：

```text
GenericWindowsAdapter
```

以后可以增加：

```text
SlackAdapter
WeChatAdapter
WhatsAppAdapter
```

但 V0.1 不实现这些 Adapter。

---

# 36. 第一版 UI

不要把大量时间花在 UI 上。

主界面只需要：

```text
Home
History
Learning
Settings
```

设计：

* 简洁
* 白色 / 深色
* 接近现代 Windows App
* 不需要复杂动画

优先保证：

```text
快捷
稳定
低干扰
```

---

# 37. 性能目标

启动：

尽量：

```text
< 2 秒
```

内存：

目标：

```text
< 150 MB
```

快捷键响应：

```text
< 150ms
```

从选中文字到出现 Loading 浮窗：

```text
< 300ms
```

AI 翻译耗时不算在 UI 响应延迟中。

---

# 38. 错误处理

如果没有选中文字：

显示：

```text
No text selected
```

如果 Clipboard 获取失败：

```text
Unable to capture text
Try screenshot translation
```

如果 API Key 不存在：

打开 Settings：

```text
Please configure your AI API key.
```

如果网络失败：

```text
Network error
Retry
```

---

# 39. 开发要求

请首先创建完整项目结构。

不要一次实现所有功能。

严格按照 Milestone 逐步开发。

每完成一个 Milestone：

1. 确保可以 Build；
2. 运行测试；
3. 输出本阶段完成内容；
4. 输出已知问题；
5. 再进入下一阶段。

不要在没有验证 Capture / Injection 的情况下提前开发复杂 Learning UI。

---

# 40. 第一阶段最重要验证

项目成功与否主要看两个技术能力：

## A

是否可以在：

```text
Slack
WeChat
WhatsApp
Chrome
Outlook
```

统一读取选中文字。

## B

是否可以在这些 App 的输入框：

```text
中文
↓
翻译
↓
英文
```

并成功写回。

第一阶段优先解决这两个问题。

---

# 41. 产品核心体验

最终用户体验必须尽量接近：

### 阅读

```text
看到英文
↓
选中
↓
Alt + Q
↓
直接看到中文
```

### 回复

```text
输入中文
↓
Alt + Enter
↓
变成英文
↓
自己按 Enter 发送
```

### 图片

```text
Alt + Shift + Q
↓
框选
↓
识别
↓
翻译
```

### 学习

```text
★
↓
保存
↓
Learning
```

不要让用户：

```text
复制
↓
切换 App
↓
打开翻译软件
↓
粘贴
↓
翻译
↓
再复制
↓
切回来
```

消除这个流程就是 LingoDesk V0.1 的核心价值。

---

# 42. Codex 第一任务

请不要立即实现完整项目。

第一步只完成一个 Windows Proof of Concept。

要求：

```text
Tauri + React
```

实现：

```text
1. 注册 Alt + Q 全局快捷键
2. 保存当前 Clipboard
3. 模拟 Ctrl+C
4. 获取当前选中文字
5. 恢复原 Clipboard
6. 弹出一个 Always-on-top 小窗口
7. 显示获取到的文字
```

暂时：

```text
不要接 AI
不要接 SQLite
不要做 OCR
不要做 Learning
```

需要验证的软件：

```text
Slack Desktop
Chrome
WeChat
WhatsApp Desktop
Notepad
```

测试完成后输出：

```text
App
是否成功
获取方式
存在的问题
```

如果此 Proof of Concept 成功，再继续 Translation Engine。

---

# 43. 最终要求

代码：

* 模块化
* 可维护
* 不把 App 名称硬编码到核心逻辑
* UI 与 Capture Engine 解耦
* AI Provider 与业务解耦
* Windows-specific Code 单独封装
* 所有敏感信息不得写入 Git Repository

优先级：

```text
兼容性
>
稳定性
>
交互速度
>
功能数量
>
视觉效果
```
