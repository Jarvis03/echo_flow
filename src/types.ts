export type CaptureErrorCode =
  | "no_text_selected"
  | "clipboard_unavailable"
  | "clipboard_restore_failed"
  | "input_blocked"
  | "unsupported_platform"
  | "capture_in_progress"
  | "unknown";

export interface CaptureResult {
  success: boolean;
  text?: string;
  source: "clipboard";
  errorCode?: CaptureErrorCode;
  message?: string;
}

export interface TranslationStarted {
  originalText: string;
  provider: string;
  mode: TranslationMode;
}

export type TranslationMode = "read" | "reply";

export interface TranslationResult {
  success: boolean;
  originalText: string;
  translatedText?: string;
  provider: string;
  errorCode?: "missing_api_key" | "network_error" | "provider_error" | "invalid_response";
  message?: string;
  mode: TranslationMode;
  injected: boolean;
}

export type CaptureState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "translating"; originalText: string; provider: string; mode: TranslationMode }
  | { status: "success"; originalText: string; translatedText: string; provider: string; mode: TranslationMode; injected: boolean }
  | { status: "error"; code?: CaptureErrorCode | TranslationResult["errorCode"]; message: string; originalText?: string };
