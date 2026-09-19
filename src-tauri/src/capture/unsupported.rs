use super::CaptureResult;

pub fn capture_selected_text() -> CaptureResult {
    CaptureResult::error(
        "unsupported_platform",
        "This proof of concept currently supports Windows only",
    )
}

pub fn capture_focused_input() -> CaptureResult {
    capture_selected_text()
}
