import { describe, expect, it } from "vitest";
import { toCaptureState, toTranslationState } from "./App";

describe("toCaptureState", () => {
  it("maps captured text to success", () => {
    expect(toCaptureState({ success: true, text: "Hello", source: "clipboard" })).toEqual({
      status: "translating",
      originalText: "Hello",
      provider: "Google Translate",
    });
  });

  it("maps a missing selection to a useful error", () => {
    expect(
      toCaptureState({
        success: false,
        source: "clipboard",
        errorCode: "no_text_selected",
        message: "No text selected",
      }),
    ).toEqual({
      status: "error",
      code: "no_text_selected",
      message: "No text selected",
    });
  });

  it("maps a Google response to translated text", () => {
    expect(
      toTranslationState({
        success: true,
        originalText: "Hello",
        translatedText: "你好",
        provider: "Google Translate",
      }),
    ).toEqual({
      status: "success",
      originalText: "Hello",
      translatedText: "你好",
      provider: "Google Translate",
    });
  });
});
