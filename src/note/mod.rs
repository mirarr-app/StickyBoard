use gtk4 as gtk;
use gtk::prelude::*;
use gdk4 as gdk;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::ipc::{send_ipc_request, IpcRequest, IpcResponse};
use crate::config::find_bin_path;
use crate::{log_error, log_info};

const NOTE_CSS: &str = "
window {
    background-color: transparent;
}
.note-container {
    border-radius: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    padding: 10px;
    background-clip: padding-box;
}
.note-yellow { background-color: #fef3c7; color: #78350f; }
.note-blue { background-color: #dbeafe; color: #1e3a8a; }
.note-green { background-color: #d1fae5; color: #065f46; }
.note-pink { background-color: #fce7f3; color: #831843; }
.note-orange { background-color: #ffedd5; color: #7c2d12; }

.note-text-view {
    background-color: transparent;
    color: inherit;
    font-family: 'Inter', 'Outfit', 'Sans', sans-serif;
    font-size: 13px;
    line-height: 1.5;
}
.note-text-view text {
    background-color: transparent;
}

.top-bar {
    margin-bottom: 4px;
}
.top-bar-btn {
    background: transparent;
    border: none;
    padding: 4px 8px;
    border-radius: 4px;
    color: inherit;
    opacity: 0.6;
    font-weight: bold;
}
.top-bar-btn:hover {
    background: rgba(0, 0, 0, 0.08);
    opacity: 0.9;
}

.color-popover-box {
    padding: 6px;
}
.color-circle {
    min-width: 24px;
    min-height: 24px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.2);
    margin: 4px;
}
.color-circle.yellow { background-color: #fef3c7; }
.color-circle.blue { background-color: #dbeafe; }
.color-circle.green { background-color: #d1fae5; }
.color-circle.pink { background-color: #fce7f3; }
.color-circle.orange { background-color: #ffedd5; }
";

struct NoteState {
    id: i64,
    text: RefCell<String>,
    color: RefCell<String>,
    pos_x: Cell<i32>,
    pos_y: Cell<i32>,
    width: Cell<i32>,
    height: Cell<i32>,
    timeout_id: RefCell<Option<glib::SourceId>>,
}

pub fn run() {
    // Parse note ID from command line arguments
    let mut note_id: Option<i64> = None;
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--id" && i + 1 < args.len() {
            if let Ok(id) = args[i + 1].parse::<i64>() {
                note_id = Some(id);
            }
        }
    }

    let id = match note_id {
        Some(val) => val,
        None => {
            eprintln!("Usage: stickyboard-note --id <note_id>");
            std::process::exit(1);
        }
    };

    let app = gtk::Application::builder()
        .application_id(format!("com.stickyboard.note{}", id))
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(move |app| {
        build_ui(app, id);
    });

    app.run_with_args(&args);
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(NOTE_CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &gtk::Application, id: i64) {
    // 1. Fetch note details from Daemon via IPC
    let req = IpcRequest::GetNote { id };
    let note = match send_ipc_request(&req) {
        Ok(IpcResponse::NoteDetails { note }) => note,
        Ok(IpcResponse::Error { message }) => {
            log_error!("Daemon returned error fetching note id={}: {}", id, message);
            std::process::exit(1);
        }
        Err(e) => {
            log_error!("IPC error fetching note id={}: {}", id, e);
            std::process::exit(1);
        }
        _ => {
            log_error!("Unexpected response from daemon fetching note id={}", id);
            std::process::exit(1);
        }
    };

    let state = Rc::new(NoteState {
        id,
        text: RefCell::new(note.text.clone()),
        color: RefCell::new(note.color.clone()),
        pos_x: Cell::new(note.pos_x),
        pos_y: Cell::new(note.pos_y),
        width: Cell::new(note.width),
        height: Cell::new(note.height),
        timeout_id: RefCell::new(None),
    });

    // Create the Window
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title(format!("stickyboard-note-{}", id))
        .default_width(state.width.get())
        .default_height(state.height.get())
        .decorated(false)
        .build();

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(vec![
            "note-container".to_string(),
            format!("note-{}", state.color.borrow()),
        ])
        .build();

    // Top Bar (controls & drag area)
    let top_bar = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_classes(vec!["top-bar".to_string()])
        .build();

    // Color button
    let color_btn = gtk::Button::builder()
        .label("🎨")
        .css_classes(vec!["top-bar-btn".to_string()])
        .build();

    // Delete button
    let delete_btn = gtk::Button::builder()
        .label("✕")
        .css_classes(vec!["top-bar-btn".to_string()])
        .build();

    // Drag spacer
    let drag_spacer = gtk::Label::builder()
        .label("")
        .hexpand(true)
        .build();

    top_bar.append(&color_btn);
    top_bar.append(&drag_spacer);
    top_bar.append(&delete_btn);
    container.append(&top_bar);

    // Text editor area
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let text_view = gtk::TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .css_classes(vec!["note-text-view".to_string()])
        .build();

    text_view.buffer().set_text(&state.text.borrow());
    scrolled.set_child(Some(&text_view));
    container.append(&scrolled);

    window.set_child(Some(&container));

    // --- Wire events ---

    // 1. Window Drag Event
    let drag_gesture = gtk::GestureClick::new();
    let window_weak = window.downgrade();
    drag_gesture.connect_pressed(move |gesture, _, _, _| {
        if gesture.current_button() == gdk::BUTTON_PRIMARY {
            if let Some(win) = window_weak.upgrade() {
                if let Some(sequence) = gesture.current_sequence() {
                    if let Some(event) = gesture.last_event(Some(&sequence)) {
                        if let Some(device) = event.device() {
                            win.begin_move_drag(
                                gdk::BUTTON_PRIMARY as i32,
                                &device,
                                Some(&sequence),
                            );
                        }
                    }
                }
            }
        }
    });
    drag_spacer.add_controller(drag_gesture);

    // 2. Color Popover picker
    let popover = gtk::Popover::new();
    let popover_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_classes(vec!["color-popover-box".to_string()])
        .build();

    let colors = vec!["yellow", "blue", "green", "pink", "orange"];
    for color_name in colors {
        let color_circle = gtk::Button::builder()
            .css_classes(vec!["color-circle".to_string(), color_name.to_string()])
            .build();

        let state_clone = state.clone();
        let container_weak = container.downgrade();
        let popover_weak = popover.downgrade();
        let text_view_weak = text_view.downgrade();

        color_circle.connect_clicked(move |_| {
            let current_color = state_clone.color.borrow().clone();
            
            // Update CSS
            if let Some(cont) = container_weak.upgrade() {
                cont.remove_css_class(&format!("note-{}", current_color));
                cont.add_css_class(&format!("note-{}", color_name));
            }

            // Update State
            state_clone.color.replace(color_name.to_string());

            // Save to DB via IPC
            let view = text_view_weak.upgrade().unwrap();
            let text = get_text_view_content(&view);
            send_update_ipc(
                state_clone.id,
                &text,
                color_name,
                state_clone.pos_x.get(),
                state_clone.pos_y.get(),
                state_clone.width.get(),
                state_clone.height.get(),
            );

            // Close popover
            if let Some(p) = popover_weak.upgrade() {
                p.popdown();
            }
        });
        popover_box.append(&color_circle);
    }
    popover.set_child(Some(&popover_box));
    popover.set_parent(&color_btn);
    color_btn.connect_clicked(move |_| {
        popover.popup();
    });

    // 3. Delete Button event
    let window_weak = window.downgrade();
    let note_id = state.id;
    delete_btn.connect_clicked(move |_| {
        let req = IpcRequest::DeleteNote { id: note_id };
        match send_ipc_request(&req) {
            Ok(IpcResponse::Ok) => {
                log_info!("Note id={} delete confirmed by daemon, closing window.", note_id);
                if let Some(win) = window_weak.upgrade() {
                    win.close();
                }
            }
            Ok(IpcResponse::Error { message }) => {
                log_error!("Failed to delete note id={} via daemon: {}", note_id, message);
            }
            Err(e) => {
                log_error!("IPC error deleting note id={}: {}", note_id, e);
            }
            _ => {}
        }
    });

    // 4. Autosave (Debounced Text changed)
    let state_clone = state.clone();
    let text_view_weak = text_view.downgrade();
    text_view.buffer().connect_changed(move |_| {
        // Cancel previous timeout
        if let Some(source_id) = state_clone.timeout_id.borrow_mut().take() {
            source_id.remove();
        }

        // Schedule new timeout (500ms debounce)
        let state_sub = state_clone.clone();
        let view_weak = text_view_weak.clone();

        let source_id = glib::timeout_add_local(
            std::time::Duration::from_millis(500),
            move || {
                if let Some(view) = view_weak.upgrade() {
                    let text = get_text_view_content(&view);
                    state_sub.text.replace(text.clone());

                    send_update_ipc(
                        state_sub.id,
                        &text,
                        &state_sub.color.borrow(),
                        state_sub.pos_x.get(),
                        state_sub.pos_y.get(),
                        state_sub.width.get(),
                        state_sub.height.get(),
                    );
                }
                glib::ControlFlow::Break
            },
        );

        state_sub.timeout_id.replace(Some(source_id));
    });

    window.present();

    // 5. Hyprland placement mapping loop
    let state_clone = state.clone();
    let mut mapping_tries = 0;
    
    // Read starting target coordinates
    let target_x = state.pos_x.get();
    let target_y = state.pos_y.get();
    let target_w = state.width.get();
    let target_h = state.height.get();

    glib::timeout_add_local(
        std::time::Duration::from_millis(50),
        move || {
            mapping_tries += 1;
            if mapping_tries > 20 {
                log_error!("Mapping timed out. Hyprland client for note id={} was not found.", id);
                return glib::ControlFlow::Break;
            }

            match crate::hyprland::position_note_window(id, target_x, target_y, target_w, target_h) {
                Ok(true) => {
                    log_info!("Note window id={} successfully positioned on Hyprland.", id);
                    
                    // Start position tracking only AFTER successful placement
                    start_position_tracking(state_clone.clone(), text_view.downgrade());
                    glib::ControlFlow::Break
                }
                Ok(false) => {
                    // Window not yet mapped in Hyprland, check again on next timeout tick
                    glib::ControlFlow::Continue
                }
                Err(e) => {
                    log_error!("Error positioning note window id={} on Hyprland: {}", id, e);
                    glib::ControlFlow::Break
                }
            }
        },
    );
}

/// Periodic job checking window position/size and updating the DB.
fn start_position_tracking(state: Rc<NoteState>, text_view_weak: glib::WeakRef<gtk::TextView>) {
    let note_id = state.id;
    
    glib::timeout_add_local(
        std::time::Duration::from_secs(1),
        move || {
            let title = format!("stickyboard-note-{}", note_id);
            if let Ok(Some(client)) = crate::hyprland::find_client_by_title(&title) {
                let curr_x = client.at[0];
                let curr_y = client.at[1];
                let curr_w = client.size[0];
                let curr_h = client.size[1];

                if curr_x != state.pos_x.get()
                    || curr_y != state.pos_y.get()
                    || curr_w != state.width.get()
                    || curr_h != state.height.get()
                {
                    log_info!("Geometry change detected for note id={}: ({}, {}) {}x{} -> ({}, {}) {}x{}",
                        note_id, state.pos_x.get(), state.pos_y.get(), state.width.get(), state.height.get(),
                        curr_x, curr_y, curr_w, curr_h
                    );

                    state.pos_x.set(curr_x);
                    state.pos_y.set(curr_y);
                    state.width.set(curr_w);
                    state.height.set(curr_h);

                    let text = if let Some(view) = text_view_weak.upgrade() {
                        get_text_view_content(&view)
                    } else {
                        state.text.borrow().clone()
                    };

                    send_update_ipc(
                        note_id,
                        &text,
                        &state.color.borrow(),
                        curr_x,
                        curr_y,
                        curr_w,
                        curr_h,
                    );
                }
            }
            glib::ControlFlow::Continue
        },
    );
}

fn get_text_view_content(text_view: &gtk::TextView) -> String {
    let buffer = text_view.buffer();
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    buffer.text(&start, &end, false).to_string()
}

fn send_update_ipc(
    id: i64,
    text: &str,
    color: &str,
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,
) {
    let req = IpcRequest::UpdateNote {
        id,
        text: text.to_string(),
        color: color.to_string(),
        pos_x,
        pos_y,
        width,
        height,
    };
    if let Err(e) = send_ipc_request(&req) {
        log_error!("Failed to send UpdateNote IPC for id={}: {}", id, e);
    }
}
