mod capture;
mod floating_window;
mod translation;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use translation::{
    GoogleTranslationProvider, TranslationProvider, TranslationResult, TranslationStarted,
};

static CAPTURE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

fn start_capture(app: AppHandle) {
    if CAPTURE_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let _ = app.emit_to("capture", "capture-started", ());
    std::thread::spawn(move || {
        let capture_result = capture::capture_selected_text();

        if let Some(window) = app.get_webview_window("capture") {
            floating_window::position_near_cursor(&window);
            let _ = window.show();

            let Some(original_text) = capture_result.text.clone() else {
                let _ = window.emit("capture-result", &capture_result);
                CAPTURE_IN_PROGRESS.store(false, Ordering::Release);
                return;
            };

            let _ = window.emit(
                "translation-started",
                TranslationStarted {
                    original_text: original_text.clone(),
                    provider: "Google Translate",
                },
            );
            eprintln!(
                "translation.start provider=google text_length={}",
                original_text.chars().count()
            );

            let translation_result = match GoogleTranslationProvider::from_environment() {
                Ok(provider) => match provider.translate_read(&original_text) {
                    Ok(translated) => {
                        eprintln!("translation.success provider=google");
                        TranslationResult::success(original_text, translated, provider.name())
                    }
                    Err(error) => {
                        eprintln!("translation.fail provider=google code={}", error.code());
                        TranslationResult::error(original_text, provider.name(), error)
                    }
                },
                Err(error) => {
                    eprintln!("translation.fail provider=google code={}", error.code());
                    TranslationResult::error(original_text, "Google Translate", error)
                }
            };
            let _ = window.emit("translation-result", &translation_result);
        }

        CAPTURE_IN_PROGRESS.store(false, Ordering::Release);
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let translate_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyQ);

    let shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_shortcut(translate_shortcut)
        .expect("failed to configure Alt+Q")
        .with_handler(move |app, shortcut, event| {
            if shortcut == &translate_shortcut && event.state() == ShortcutState::Released {
                start_capture(app.clone());
            }
        })
        .build();

    tauri::Builder::default()
        .plugin(shortcut_plugin)
        .run(tauri::generate_context!())
        .expect("error while running LingoDesk");
}
