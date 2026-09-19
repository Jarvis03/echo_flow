use tauri::{PhysicalPosition, WebviewWindow};

const CURSOR_GAP: i32 = 16;

pub fn position_near_cursor(window: &WebviewWindow) {
    let Ok(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(window_size) = window.outer_size() else {
        return;
    };
    let Ok(monitors) = window.available_monitors() else {
        return;
    };

    let cursor_x = cursor.x.round() as i32;
    let cursor_y = cursor.y.round() as i32;
    let width = window_size.width as i32;
    let height = window_size.height as i32;

    let monitor = monitors.iter().find(|monitor| {
        let origin = monitor.position();
        let size = monitor.size();
        cursor_x >= origin.x
            && cursor_x < origin.x + size.width as i32
            && cursor_y >= origin.y
            && cursor_y < origin.y + size.height as i32
    });

    let mut x = cursor_x + CURSOR_GAP;
    let mut y = cursor_y + CURSOR_GAP;

    if let Some(monitor) = monitor {
        let origin = monitor.position();
        let size = monitor.size();
        let right = origin.x + size.width as i32;
        let bottom = origin.y + size.height as i32;

        if x + width > right {
            x = cursor_x - width - CURSOR_GAP;
        }
        if y + height > bottom {
            y = cursor_y - height - CURSOR_GAP;
        }
        x = x.max(origin.x);
        y = y.max(origin.y);
    }

    let _ = window.set_position(PhysicalPosition::new(x, y));
}
