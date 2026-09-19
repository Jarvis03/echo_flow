import { useEffect, useState } from "react";
import { Check, Clipboard, Command, X } from "lucide-react";
import { hideWindow, listenForCapture } from "./lib/desktop";
import type { CaptureResult, CaptureState, TranslationResult } from "./types";

function getInitialState(): CaptureState {
  if (!import.meta.env.DEV) return { status: "loading" };

  const demo = new URLSearchParams(window.location.search).get("demo");
  if (demo === "success") {
    return {
      status: "success",
      originalText: "Could you verify the GPS data before the release?",
      translatedText: "你能在发布前确认一下 GPS 数据吗？",
      provider: "Google Translate",
    };
  }
  if (demo === "translating") {
    return {
      status: "translating",
      originalText: "Could you verify the GPS data before the release?",
      provider: "Google Translate",
    };
  }
  return { status: "idle" };
}

function toCaptureState(result: CaptureResult): CaptureState {
  if (result.success && result.text) {
    return { status: "translating", originalText: result.text, provider: "Google Translate" };
  }

  return {
    status: "error",
    code: result.errorCode,
    message: result.message ?? "Unable to capture text",
  };
}

function toTranslationState(result: TranslationResult): CaptureState {
  if (result.success && result.translatedText) {
    return {
      status: "success",
      originalText: result.originalText,
      translatedText: result.translatedText,
      provider: result.provider,
    };
  }

  return {
    status: "error",
    code: result.errorCode,
    message: result.message ?? "Translation failed",
    originalText: result.originalText,
  };
}

export default function App() {
  const [state, setState] = useState<CaptureState>(getInitialState);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    let cancelled = false;

    listenForCapture(
      () => {
        setCopied(false);
        setState({ status: "loading" });
      },
      (result) => setState(toCaptureState(result)),
      (result) => {
        setState({
          status: "translating",
          originalText: result.originalText,
          provider: result.provider,
        });
      },
      (result) => setState(toTranslationState(result)),
    ).then((unlisten) => {
      if (cancelled) unlisten();
      else dispose = unlisten;
    });

    return () => {
      cancelled = true;
      dispose?.();
    };
  }, []);

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") void hideWindow();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, []);

  const copyText = async () => {
    if (state.status !== "success") return;
    await navigator.clipboard.writeText(state.translatedText);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  };

  return (
    <main className="shell" aria-live="polite">
      <header className="titlebar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <span className="brand-mark" aria-hidden="true">
            <Command size={14} strokeWidth={2.4} />
          </span>
          <span data-tauri-drag-region>LingoDesk</span>
          <span className="badge">PoC</span>
        </div>
        <button className="icon-button" aria-label="关闭" onClick={() => void hideWindow()}>
          <X size={16} />
        </button>
      </header>

      <section className={`content content-${state.status}`}>
        {state.status === "idle" && (
          <div className="empty-state">
            <div className="keycap-row" aria-hidden="true">
              <kbd>Alt</kbd><span>+</span><kbd>Q</kbd>
            </div>
            <h1>选择文字，立即捕获</h1>
            <p>在任意应用中选中文字，然后按快捷键。</p>
          </div>
        )}

        {state.status === "loading" && (
          <div className="loading-state">
            <span className="spinner" aria-hidden="true" />
            <div>
              <h1>正在获取文字…</h1>
              <p>请保持原窗口的选择状态。</p>
            </div>
          </div>
        )}

        {state.status === "translating" && (
          <div className="translation-layout">
            <div className="section-label">
              <span>原文</span>
              <span className="source-pill">{state.provider}</span>
            </div>
            <div className="original-text">{state.originalText}</div>
            <div className="translating-row">
              <span className="spinner spinner-small" aria-hidden="true" />
              <span>正在翻译为中文…</span>
            </div>
          </div>
        )}

        {state.status === "success" && (
          <div className="translation-layout">
            <div className="section-label">
              <span>原文</span>
              <span className="source-pill">{state.provider}</span>
            </div>
            <div className="original-text">{state.originalText}</div>
            <div className="translation-divider" />
            <div className="section-label"><span>中文翻译</span></div>
            <div className="translated-text" tabIndex={0}>{state.translatedText}</div>
          </div>
        )}

        {state.status === "error" && (
          <div className="error-state">
            <span className="error-icon">!</span>
            <div>
              <h1>{state.message}</h1>
              <p>
                {state.code === "no_text_selected"
                  ? "请先选择一段文字，再按 Alt + Q。"
                  : state.code === "missing_api_key"
                    ? "设置环境变量后重新启动 LingoDesk。"
                  : "请回到目标应用后重试。"}
              </p>
            </div>
          </div>
        )}
      </section>

      <footer className="footer">
        <span className="privacy-note">
          {state.status === "idle" || state.status === "loading"
            ? "剪贴板内容会自动恢复"
            : "翻译文本发送至 Google Cloud"}
        </span>
        {state.status === "success" && (
          <button className="copy-button" onClick={() => void copyText()}>
            {copied ? <Check size={15} /> : <Clipboard size={15} />}
            {copied ? "已复制" : "复制"}
          </button>
        )}
      </footer>
    </main>
  );
}

export { toCaptureState, toTranslationState };
