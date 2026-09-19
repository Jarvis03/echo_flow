# LingoDesk Windows PoC

This repository implements the Windows capture proof of concept plus the first
English-to-Chinese translation milestone requested by `echoflow.md`:

- `Alt + Q` global shortcut
- preserve the current Windows clipboard
- send `Ctrl + C` to the foreground application
- read selected Unicode text
- restore the original clipboard
- show the result in a small always-on-top window
- translate captured text to Simplified Chinese with Google Cloud Translation

SQLite, OCR, Learning, text injection, and app-specific adapters remain out of
scope for this milestone.

## Requirements

- Windows 10 or 11
- Node.js 20+
- Rust 1.77.2+ with the MSVC toolchain
- Microsoft Edge WebView2 runtime
- Visual Studio Build Tools with **Desktop development with C++**

## Run

Create or select a Google Cloud project, enable **Cloud Translation API**, and
create an API key restricted to that API. The key is read only from the process
environment and must never be committed:

```powershell
$env:GOOGLE_TRANSLATE_API_KEY="your-api-key"
npm install
npm run tauri dev
```

Select text in another application and press `Alt + Q`. LingoDesk waits for
the hotkey keys to be released, captures the selection, restores the previous
clipboard data, translates the selection, and opens the floating window near
the pointer.

## Validate

Before each test, copy a recognizable text or image to the clipboard. After
capturing a selection, paste elsewhere to confirm that the original clipboard
content was restored.

| App | Result | Capture method | Known issue |
| --- | --- | --- | --- |
| Slack Desktop | Not run | Clipboard | Requires manual Windows validation |
| Chrome | Not run | Clipboard | Requires manual Windows validation |
| WeChat | Not run | Clipboard | Requires manual Windows validation |
| WhatsApp Desktop | Not run | Clipboard | Requires manual Windows validation |
| Notepad | Not run | Clipboard | Requires manual Windows validation |

Windows can block simulated input across privilege boundaries. Run LingoDesk
at the same integrity level as the target application.
