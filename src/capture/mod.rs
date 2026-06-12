use gtk4 as gtk;
use gtk::prelude::*;
use gdk4 as gdk;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ipc::{send_ipc_request, IpcRequest, IpcResponse};
use crate::config::{DEFAULT_COLOR, DEFAULT_HEIGHT, DEFAULT_WIDTH};

const CAPTURE_CSS: &str = "
window {
    background-color: transparent;
}
.capture-container {
    background-color: rgba(24, 24, 27, 0.95);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3), 0 8px 10px -6px rgba(0, 0, 0, 0.3);
    padding: 16px;
}
.capture-text-view {
    background-color: transparent;
    color: #f4f4f5;
    font-family: 'Inter', 'Outfit', 'Sans', sans-serif;
    font-size: 14px;
    line-height: 1.5;
}
.capture-text-view text {
    background-color: transparent;
}
.capture-hint {
    color: #a1a1aa;
    font-family: 'Inter', sans-serif;
    font-size: 11px;
    margin-top: 8px;
}
";

pub fn run() {
    let app = gtk::Application::builder()
        .application_id("com.stickyboard.capture")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(|app| {
        build_ui(app);
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CAPTURE_CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("StickyBoard - Capture Note")
        .default_width(400)
        .default_height(200)
        .resizable(false)
        .decorated(false)
        .build();

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(vec!["capture-container".to_string()])
        .build();

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let text_view = gtk::TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .css_classes(vec!["capture-text-view".to_string()])
        .placeholder_text("Type a quick note...")
        .build();

    scrolled_window.set_child(Some(&text_view));
    container.append(&scrolled_window);

    let hint_label = gtk::Label::builder()
        .label("Enter to save  •  Shift+Enter for newline  •  Esc to cancel")
        .css_classes(vec!["capture-hint".to_string()])
        .halign(gtk::Align::Center)
        .build();
    container.append(&hint_label);

    window.set_child(Some(&container));

    // Listen to key events
    let key_controller = gtk::EventControllerKey::new();
    
    let window_weak = window.downgrade();
    let text_view_weak = text_view.downgrade();
    
    key_controller.connect_key_pressed(move |_, keyval, _, state| {
        let win = match window_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };
        let view = match text_view_weak.upgrade() {
            Some(v) => v,
            None => return glib::Propagation::Proceed,
        };

        if keyval == gdk::Key::Escape {
            win.close();
            glib::Propagation::Stop
        } else if keyval == gdk::Key::Return || keyval == gdk::Key::KP_Enter {
            let has_shift = state.contains(gdk::ModifierType::SHIFT_MASK);
            if !has_shift {
                // Save note
                let buffer = view.buffer();
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let text = buffer.text(&start, &end, false).to_string();
                let trimmed = text.trim();

                if !trimmed.is_empty() {
                    let req = IpcRequest::NewNote {
                        text: trimmed.to_string(),
                        color: DEFAULT_COLOR.to_string(),
                        pos_x: 100, // Daemon cascading offsets will trigger
                        pos_y: 100,
                        width: DEFAULT_WIDTH,
                        height: DEFAULT_HEIGHT,
                    };

                    match send_ipc_request(&req) {
                        Ok(IpcResponse::NewNoteCreated { id }) => {
                            println!("Note created successfully: {}", id);
                            win.close();
                        }
                        Ok(IpcResponse::Error { message }) => {
                            show_error_dialog(&win, "Daemon Error", &message);
                        }
                        Err(e) => {
                            show_error_dialog(
                                &win,
                                "IPC Connection Error",
                                &format!("Is stickyboard-daemon running?\n\nDetails: {}", e),
                            );
                        }
                        _ => {
                            show_error_dialog(&win, "Error", "Unexpected response from daemon.");
                        }
                    }
                } else {
                    win.close(); // Empty note, just close
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        } else {
            glib::Propagation::Proceed
        }
    });

    text_view.add_controller(key_controller);

    window.present();
    
    // Grab focus on the text area
    text_view.grab_focus();
}

fn show_error_dialog(window: &gtk::ApplicationWindow, title: &str, message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(gtk::MessageType::Error)
        .buttons(gtk::ButtonsType::Ok)
        .text(title)
        .secondary_text(message)
        .build();

    let window_clone = window.clone();
    dialog.connect_response(move |d, _| {
        d.destroy();
        window_clone.close();
    });
    dialog.present();
}
