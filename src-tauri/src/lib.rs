mod capture;
mod floating_window;
mod injection;
mod translation;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use translation::{
    GoogleTranslationProvider, TranslationProvider, TranslationResult, TranslationStarted,
};

static OPERATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

fn start_capture(app: AppHandle) {
    if OPERATION_IN_PROGRESS.swap(true, Ordering::AcqRel) {
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
                OPERATION_IN_PROGRESS.store(false, Ordering::Release);
                return;
            };

            let _ = window.emit(
                "translation-started",
                TranslationStarted {
                    original_text: original_text.clone(),
                    provider: "Google Translate",
                    mode: "read",
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
                        TranslationResult::success(
                            original_text,
                            translated,
                            provider.name(),
                            "read",
                            false,
                        )
                    }
                    Err(error) => {
                        eprintln!("translation.fail provider=google code={}", error.code());
                        TranslationResult::error(original_text, provider.name(), "read", error)
                    }
                },
                Err(error) => {
                    eprintln!("translation.fail provider=google code={}", error.code());
                    TranslationResult::error(original_text, "Google Translate", "read", error)
                }
            };
            let _ = window.emit("translation-result", &translation_result);
        }

        OPERATION_IN_PROGRESS.store(false, Ordering::Release);
    });
}

fn start_reply(app: AppHandle) {
    if OPERATION_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let target = injection::foreground_target();
    let _ = app.emit_to("capture", "capture-started", ());
    std::thread::spawn(move || {
        let capture_result = capture::capture_focused_input();

        if let Some(window) = app.get_webview_window("capture") {
            floating_window::position_near_cursor(&window);
            let _ = window.show();

            let Some(original_text) = capture_result.text.clone() else {
                let _ = window.emit("capture-result", &capture_result);
                OPERATION_IN_PROGRESS.store(false, Ordering::Release);
                return;
            };

            let _ = window.emit(
                "translation-started",
                TranslationStarted {
                    original_text: original_text.clone(),
                    provider: "Google Translate",
                    mode: "reply",
                },
            );
            eprintln!(
                "translation.start provider=google mode=reply text_length={}",
                original_text.chars().count()
            );

            let translation_result = match GoogleTranslationProvider::from_environment() {
                Ok(provider) => match provider.translate_reply(&original_text) {
                    Ok(translated) if injection::type_text(target, &translated) => {
                        eprintln!("translation.success provider=google mode=reply injected=true");
                        TranslationResult::success(
                            original_text,
                            translated,
                            provider.name(),
                            "reply",
                            true,
                        )
                    }
                    Ok(translated) => {
                        eprintln!("inject.fail code=input_blocked");
                        TranslationResult::success(
                            original_text,
                            translated,
                            provider.name(),
                            "reply",
                            false,
                        )
                    }
                    Err(error) => {
                        eprintln!(
                            "translation.fail provider=google mode=reply code={}",
                            error.code()
                        );
                        TranslationResult::error(original_text, provider.name(), "reply", error)
                    }
                },
                Err(error) => {
                    eprintln!(
                        "translation.fail provider=google mode=reply code={}",
                        error.code()
                    );
                    TranslationResult::error(original_text, "Google Translate", "reply", error)
                }
            };
            let _ = window.emit("translation-result", &translation_result);
        }

        OPERATION_IN_PROGRESS.store(false, Ordering::Release);
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let translate_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyQ);
    let reply_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyA);

    let shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts([translate_shortcut, reply_shortcut])
        .expect("failed to configure global shortcuts")
        .with_handler(move |app, shortcut, event| {
            if event.state() == ShortcutState::Released {
                if shortcut == &translate_shortcut {
                    start_capture(app.clone());
                } else if shortcut == &reply_shortcut {
                    start_reply(app.clone());
                }
            }
        })
        .build();

    tauri::Builder::default()
        .plugin(shortcut_plugin)
        .run(tauri::generate_context!())
        .expect("error while running LingoDesk");
}
