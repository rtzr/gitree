use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use owo_colors::OwoColorize;

/// Animated "scanning..." indicator drawn on stderr.
///
/// Becomes a no-op when stderr isn't a TTY (e.g. piped output) so it
/// never pollutes captured logs.
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    active: bool,
}

impl Spinner {
    pub fn start(message: String, color: bool, emoji: bool) -> Self {
        let active = std::io::stderr().is_terminal();
        if !active {
            return Self {
                running: Arc::new(AtomicBool::new(false)),
                handle: None,
                active: false,
            };
        }

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let frames: &[&str] = if emoji {
            &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
        } else {
            &["|", "/", "-", "\\"]
        };

        let handle = thread::spawn(move || {
            let mut stderr = std::io::stderr().lock();
            let _ = write!(stderr, "\x1b[?25l"); // hide cursor
            let _ = stderr.flush();

            let mut i = 0usize;
            while running_clone.load(Ordering::Relaxed) {
                let frame = frames[i % frames.len()];
                let painted_frame = if color {
                    frame.bright_cyan().to_string()
                } else {
                    frame.to_string()
                };
                let painted_msg = if color {
                    message.dimmed().to_string()
                } else {
                    message.clone()
                };
                let _ = write!(stderr, "\r{} {}", painted_frame, painted_msg);
                let _ = stderr.flush();
                thread::sleep(Duration::from_millis(80));
                i += 1;
            }

            let _ = write!(stderr, "\r\x1b[K\x1b[?25h"); // clear line + show cursor
            let _ = stderr.flush();
        });

        Self {
            running,
            handle: Some(handle),
            active: true,
        }
    }

    pub fn stop(mut self) {
        if !self.active {
            return;
        }
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        // Ensure the terminal is left in a sane state even on panic.
        if self.active && self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);
            if let Some(h) = self.handle.take() {
                let _ = h.join();
            }
        }
    }
}
