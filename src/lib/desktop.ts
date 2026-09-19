import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CaptureResult, TranslationResult, TranslationStarted } from "../types";

const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function listenForCapture(
  onStarted: () => void,
  onCaptureResult: (result: CaptureResult) => void,
  onTranslationStarted: (result: TranslationStarted) => void,
  onTranslationResult: (result: TranslationResult) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => undefined;
  }

  const unlistenStarted = await listen("capture-started", onStarted);
  const unlistenResult = await listen<CaptureResult>("capture-result", (event) => {
    onCaptureResult(event.payload);
  });
  const unlistenTranslationStarted = await listen<TranslationStarted>(
    "translation-started",
    (event) => onTranslationStarted(event.payload),
  );
  const unlistenTranslationResult = await listen<TranslationResult>(
    "translation-result",
    (event) => onTranslationResult(event.payload),
  );

  return () => {
    unlistenStarted();
    unlistenResult();
    unlistenTranslationStarted();
    unlistenTranslationResult();
  };
}

export async function hideWindow(): Promise<void> {
  if (isTauri()) {
    await getCurrentWindow().hide();
  }
}
