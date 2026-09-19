use std::{
    thread,
    time::{Duration, Instant},
};

use windows::{
    core::{Error as WindowsError, HRESULT},
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL},
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
                OpenClipboard, SetClipboardData,
            },
            Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE},
            Ole::{OleFlushClipboard, OleGetClipboard, OleSetClipboard},
        },
    },
};
use winput::{Action, Input, Vk};

use super::CaptureResult;

const CF_UNICODETEXT: u32 = 13;
const OLE_E_BLANK: HRESULT = HRESULT(0x8004_0007_u32 as i32);
const COPY_TIMEOUT: Duration = Duration::from_millis(750);

struct ComApartment;

impl ComApartment {
    fn initialize() -> Result<Self, WindowsError> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Result<Self, WindowsError> {
        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Ok(Self);
            }
            if Instant::now() >= deadline {
                return Err(WindowsError::from_thread());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

/// Captures selected text without retaining or logging its contents.
///
/// The previous clipboard is retained as an OLE IDataObject, Ctrl+C is sent to
/// the foreground app, the Unicode text is read, and the original IDataObject
/// is put back and flushed before this function returns.
pub fn capture_selected_text() -> CaptureResult {
    capture_text(false)
}

/// Selects and captures all text in the focused input control. The selection is
/// deliberately left active so the translated reply can replace it in-place.
pub fn capture_focused_input() -> CaptureResult {
    capture_text(true)
}

fn capture_text(select_all: bool) -> CaptureResult {
    let started_at = Instant::now();
    let operation = if select_all { "reply" } else { "read" };
    eprintln!("capture.clipboard.start mode={operation}");

    let result = capture_inner(select_all);
    match &result {
        Ok(text) => eprintln!(
            "capture.clipboard.success text_length={} elapsed_ms={}",
            text.chars().count(),
            started_at.elapsed().as_millis()
        ),
        Err(error) => eprintln!("capture.clipboard.fail code={}", error.code()),
    }

    match result {
        Ok(text) => CaptureResult::success(text),
        Err(error) => CaptureResult::error(error.code(), error.user_message()),
    }
}

fn capture_inner(select_all: bool) -> Result<String, CaptureError> {
    let _com = ComApartment::initialize().map_err(|_| CaptureError::ClipboardUnavailable)?;

    wait_for_hotkey_release(select_all);

    let original = match unsafe { OleGetClipboard() } {
        Ok(data) => Some(data),
        Err(error) if error.code() == OLE_E_BLANK => None,
        Err(_) => return Err(CaptureError::ClipboardUnavailable),
    };
    let original_text = read_unicode_text_raw().ok();

    let initial_sequence = unsafe { GetClipboardSequenceNumber() };
    if !send_copy_shortcut(select_all) {
        restore_clipboard(original.as_ref(), original_text.as_deref())?;
        return Err(CaptureError::InputBlocked);
    }

    let clipboard_changed = wait_for_clipboard_change(initial_sequence);
    let captured = if clipboard_changed {
        read_unicode_text()
    } else {
        Err(CaptureError::NoTextSelected)
    };

    // Restoration happens on every path after Ctrl+C, including empty selection
    // and clipboard read errors.
    restore_clipboard(original.as_ref(), original_text.as_deref())?;
    captured
}

fn wait_for_hotkey_release(reply_shortcut: bool) {
    let deadline = Instant::now() + Duration::from_millis(500);
    while (Vk::Alt.is_down()
        || if reply_shortcut {
            Vk::A.is_down()
        } else {
            Vk::Q.is_down()
        })
        && Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(5));
    }
}

fn send_copy_shortcut(select_all: bool) -> bool {
    let mut inputs = Vec::with_capacity(if select_all { 8 } else { 4 });
    if select_all {
        inputs.extend([
            Input::from_vk(Vk::Control, Action::Press),
            Input::from_vk(Vk::A, Action::Press),
            Input::from_vk(Vk::A, Action::Release),
            Input::from_vk(Vk::Control, Action::Release),
        ]);
        thread::sleep(Duration::from_millis(25));
    }
    inputs.extend([
        Input::from_vk(Vk::Control, Action::Press),
        Input::from_vk(Vk::C, Action::Press),
        Input::from_vk(Vk::C, Action::Release),
        Input::from_vk(Vk::Control, Action::Release),
    ]);
    winput::send_inputs(&inputs) == inputs.len() as u32
}

fn wait_for_clipboard_change(initial_sequence: u32) -> bool {
    let deadline = Instant::now() + COPY_TIMEOUT;
    while Instant::now() < deadline {
        if unsafe { GetClipboardSequenceNumber() } != initial_sequence {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

fn read_unicode_text() -> Result<String, CaptureError> {
    let text = read_unicode_text_raw()?;
    if text.trim().is_empty() {
        Err(CaptureError::NoTextSelected)
    } else {
        Ok(text)
    }
}

fn read_unicode_text_raw() -> Result<String, CaptureError> {
    let _clipboard = ClipboardGuard::open().map_err(|_| CaptureError::ClipboardUnavailable)?;
    let handle =
        unsafe { GetClipboardData(CF_UNICODETEXT) }.map_err(|_| CaptureError::NoTextSelected)?;
    let memory = HGLOBAL(handle.0);
    let size_bytes = unsafe { GlobalSize(memory) };
    if size_bytes < 2 {
        return Err(CaptureError::NoTextSelected);
    }

    let pointer = unsafe { GlobalLock(memory) } as *const u16;
    if pointer.is_null() {
        return Err(CaptureError::ClipboardUnavailable);
    }

    let max_len = size_bytes / std::mem::size_of::<u16>();
    let words = unsafe { std::slice::from_raw_parts(pointer, max_len) };
    let text_len = words.iter().position(|word| *word == 0).unwrap_or(max_len);
    let decoded = String::from_utf16(&words[..text_len]);
    let _ = unsafe { GlobalUnlock(memory) };
    let text = decoded.map_err(|_| CaptureError::ClipboardUnavailable)?;

    Ok(text)
}

fn restore_clipboard(
    original: Option<&windows::Win32::System::Com::IDataObject>,
    original_text: Option<&str>,
) -> Result<(), CaptureError> {
    let ole_restored = unsafe {
        OleSetClipboard(original)
            .and_then(|_| OleFlushClipboard())
            .is_ok()
    };
    if ole_restored {
        return Ok(());
    }

    eprintln!("capture.clipboard.restore_ole_failed fallback=text");
    match (original, original_text) {
        (_, Some(text)) => restore_unicode_text(text),
        (None, None) => clear_clipboard(),
        (Some(_), None) => Err(CaptureError::ClipboardRestoreFailed),
    }
}

fn clear_clipboard() -> Result<(), CaptureError> {
    let _clipboard = ClipboardGuard::open().map_err(|_| CaptureError::ClipboardRestoreFailed)?;
    unsafe { EmptyClipboard() }.map_err(|_| CaptureError::ClipboardRestoreFailed)
}

fn restore_unicode_text(text: &str) -> Result<(), CaptureError> {
    let encoded = text
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let byte_len = encoded.len() * std::mem::size_of::<u16>();
    let _clipboard = ClipboardGuard::open().map_err(|_| CaptureError::ClipboardRestoreFailed)?;

    unsafe { EmptyClipboard() }.map_err(|_| CaptureError::ClipboardRestoreFailed)?;
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, byte_len) }
        .map_err(|_| CaptureError::ClipboardRestoreFailed)?;
    let pointer = unsafe { GlobalLock(memory) } as *mut u16;
    if pointer.is_null() {
        let _ = unsafe { GlobalFree(Some(memory)) };
        return Err(CaptureError::ClipboardRestoreFailed);
    }

    unsafe { std::ptr::copy_nonoverlapping(encoded.as_ptr(), pointer, encoded.len()) };
    let _ = unsafe { GlobalUnlock(memory) };

    match unsafe { SetClipboardData(CF_UNICODETEXT, Some(HANDLE(memory.0))) } {
        Ok(_) => Ok(()),
        Err(_) => {
            let _ = unsafe { GlobalFree(Some(memory)) };
            Err(CaptureError::ClipboardRestoreFailed)
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CaptureError {
    NoTextSelected,
    ClipboardUnavailable,
    ClipboardRestoreFailed,
    InputBlocked,
}

impl CaptureError {
    fn code(self) -> &'static str {
        match self {
            Self::NoTextSelected => "no_text_selected",
            Self::ClipboardUnavailable => "clipboard_unavailable",
            Self::ClipboardRestoreFailed => "clipboard_restore_failed",
            Self::InputBlocked => "input_blocked",
        }
    }

    fn user_message(self) -> &'static str {
        match self {
            Self::NoTextSelected => "No text selected",
            Self::ClipboardUnavailable => "Unable to capture text",
            Self::ClipboardRestoreFailed => "Unable to restore clipboard",
            Self::InputBlocked => "Windows blocked keyboard input",
        }
    }
}
