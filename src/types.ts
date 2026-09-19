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
}

export interface TranslationResult {
  success: boolean;
  originalText: string;
  translatedText?: string;
  provider: string;
  errorCode?: "missing_api_key" | "network_error" | "provider_error" | "invalid_response";
  message?: string;
}

export type CaptureState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "translating"; originalText: string; provider: string }
  | { status: "success"; originalText: string; translatedText: string; provider: string }
  | { status: "error"; code?: CaptureErrorCode | TranslationResult["errorCode"]; message: string; originalText?: string };
