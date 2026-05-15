use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use owo_colors::OwoColorize;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Animated "scanning..." indicator drawn on stderr with a live status
/// message that the caller can update as work progresses.
///
/// Becomes a no-op when stderr isn't a TTY (e.g. piped output) so it
/// never pollutes captured logs.
pub struct Spinner {
    running: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    handle: Option<JoinHandle<()>>,
    active: bool,
}

impl Spinner {
    pub fn start(initial: String, color: bool, emoji: bool) -> Self {
        let status = Arc::new(Mutex::new(initial));
        let running = Arc::new(AtomicBool::new(true));

        let active = std::io::stderr().is_terminal();
        if !active {
            return Self {
                running,
                status,
                handle: None,
                active: false,
            };
        }

        let running_c = running.clone();
        let status_c = status.clone();

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
            while running_c.load(Ordering::Relaxed) {
                let frame = frames[i % frames.len()];
                let msg = status_c
                    .lock()
                    .map(|g| g.clone())
                    .unwrap_or_default();

                // Reserve: frame (1 cell) + space (1) + message + a couple
                // of spare cells so the cursor never lands on the last cell.
                let term_w = terminal_size::terminal_size()
                    .map(|(w, _)| w.0 as usize)
                    .unwrap_or(80);
                let max_msg_w = term_w.saturating_sub(4);
                let msg = truncate_to_width(&msg, max_msg_w);

                let painted_frame = if color {
                    frame.bright_cyan().to_string()
                } else {
                    frame.to_string()
                };
                let painted_msg = if color {
                    msg.dimmed().to_string()
                } else {
                    msg.clone()
                };
                // \r\x1b[K clears the line before drawing — without it,
                // a shorter message would leave trailing chars from the
                // previous longer one visible.
                let _ = write!(stderr, "\r\x1b[K{} {}", painted_frame, painted_msg);
                let _ = stderr.flush();
                thread::sleep(Duration::from_millis(80));
                i += 1;
            }

            let _ = write!(stderr, "\r\x1b[K\x1b[?25h"); // clear line + show cursor
            let _ = stderr.flush();
        });

        Self {
            running,
            status,
            handle: Some(handle),
            active: true,
        }
    }

    /// Update the message shown next to the spinner. Cheap to call frequently
    /// — the spinner thread only reads at frame intervals (~80ms).
    pub fn set_status(&self, s: String) {
        if let Ok(mut g) = self.status.lock() {
            *g = s;
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

/// Truncate `s` so it occupies at most `max_w` display cells. If truncation
/// happens, an ellipsis is prepended (keep the tail — the deeper path is
/// usually the more interesting part during a recursive scan).
fn truncate_to_width(s: &str, max_w: usize) -> String {
    if max_w == 0 {
        return String::new();
    }
    let w = UnicodeWidthStr::width(s);
    if w <= max_w {
        return s.to_string();
    }
    // Reserve 1 cell for the leading ellipsis.
    let budget = max_w.saturating_sub(1);
    let mut tail: Vec<char> = Vec::new();
    let mut acc = 0usize;
    for c in s.chars().rev() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if acc + cw > budget {
            break;
        }
        tail.push(c);
        acc += cw;
    }
    tail.reverse();
    let mut out = String::with_capacity(tail.len() + 1);
    out.push('…');
    out.extend(tail);
    out
}
