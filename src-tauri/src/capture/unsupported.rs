use super::CaptureResult;

pub fn capture_selected_text() -> CaptureResult {
    CaptureResult::error(
        "unsupported_platform",
        "This proof of concept currently supports Windows only",
    )
}
