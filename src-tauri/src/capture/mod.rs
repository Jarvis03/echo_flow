#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    pub success: bool,
    pub text: Option<String>,
    pub source: &'static str,
    pub error_code: Option<&'static str>,
    pub message: Option<String>,
}

impl CaptureResult {
    pub fn success(text: String) -> Self {
        Self {
            success: true,
            text: Some(text),
            source: "clipboard",
            error_code: None,
            message: None,
        }
    }

    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            success: false,
            text: None,
            source: "clipboard",
            error_code: Some(code),
            message: Some(message.into()),
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub use unsupported::capture_selected_text;
#[cfg(target_os = "windows")]
pub use windows::capture_selected_text;
