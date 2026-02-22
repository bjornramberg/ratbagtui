use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::sync::mpsc;

pub fn find_mouse_device() -> Option<String> {
    let by_id = Path::new("/dev/input/by-id");
    if let Ok(entries) = std::fs::read_dir(by_id) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("Logitech") && name.contains("hidraw") {
                if let Ok(resolved) = std::fs::canonicalize(entry.path()) {
                    return Some(resolved.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

pub async fn start_input_listener(path: String, tx: mpsc::Sender<u16>) {
    tokio::task::spawn_blocking(move || {
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open hidraw device: {}", e);
                return;
            }
        };

        let mut buf = [0u8; 64];
        let mut prev_buttons = 0u8;

        loop {
            match file.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let buttons = if n > 1 { buf[1] } else { buf[0] };
                    let pressed = buttons & !prev_buttons;
                    prev_buttons = buttons;

                    for bit in 0..8u16 {
                        if pressed & (1 << bit) != 0 {
                            let button_num = bit + 1;
                            if tx.blocking_send(button_num).is_err() {
                                return;
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}