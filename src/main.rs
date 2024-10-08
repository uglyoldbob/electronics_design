//! This package is software for designing electronic schematics and associated circuit boards.

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

/// The main font to use for schematics and board layout
const COMPUTER_MODERN_FONT: &[u8] = include_bytes!("./cmunbtl.ttf");

mod main_common;
use main_common::*;

mod component;
mod general;
mod ipc;
mod library;
mod schematic;
mod symbol;

use std::collections::HashMap;

/// Macro generated code
pub mod egui_multiwin_dynamic {
    egui_multiwin::tracked_window!(
        crate::MyApp,
        crate::ipc::IpcMessage,
        crate::window::Windows
    );
    egui_multiwin::multi_window!(
        crate::MyApp,
        crate::ipc::IpcMessage,
        crate::window::Windows
    );
}

use egui_multiwin_dynamic::multi_window::{MultiWindow, NewWindowRequest};

use crate::schematic::SchematicHolder;

mod window;

fn main() {
    let instance = single_instance::SingleInstance::new(PACKAGE_NAME).unwrap();

    let args: Vec<String> = std::env::args().collect();

    let dirs = DIRS.clone();
    let mut ac = MyApp {
        schematic: None,
        libraries: HashMap::new(),
        library_log: undo::Record::new(),
        units: general::DisplayMode::Inches,
        dirs,
        args,
    };

    let mut ipcname = String::new();
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let prefix = if let Some(dirs) = &ac.dirs {
            dirs.cache_dir().to_str().unwrap()
        } else {
            ""
        };
        ipcname.push_str(&format!("{}/{}", prefix, PACKAGE_NAME));
        let _e = std::fs::DirBuilder::new().recursive(true).create(&prefix);
    }
    #[cfg(target_os = "windows")]
    {
        ipcname.push_str(PACKAGE_NAME);
    }

    if !instance.is_single() {
        let ipc_sender = interprocess::local_socket::LocalSocketStream::connect(ipcname).unwrap();
        bincode::serialize_into(ipc_sender, &ipc::IpcMessage::NewLibrary).unwrap();
        return;
    }

    let mut event_loop = egui_multiwin::winit::event_loop::EventLoopBuilder::with_user_event();
    #[cfg(target_os = "linux")]
    egui_multiwin::winit::platform::x11::EventLoopBuilderExtX11::with_x11(&mut event_loop);
    let event_loop = event_loop.build().unwrap();
    let proxy = event_loop.create_proxy();

    std::thread::spawn(move || {
        let proxy = proxy.clone();
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let _e = std::fs::remove_file(&ipcname);
        let ipc_listener = interprocess::local_socket::LocalSocketListener::bind(ipcname);
        match ipc_listener {
            Err(e) => {
                println!("Error opening ipc {:?}", e);
            }
            Ok(ipc_listener) => {
                for mut i in ipc_listener.incoming().flatten() {
                    let proxy = proxy.clone();
                    std::thread::spawn(move || loop {
                        let msg = bincode::deserialize_from::<
                            &mut interprocess::local_socket::LocalSocketStream,
                            ipc::IpcMessage,
                        >(&mut i);
                        if let Ok(msg) = msg {
                            println!("Received a {:?}", msg);
                            proxy.send_event(msg).ok();
                        } else {
                            break;
                        }
                    });
                }
            }
        }
    });

    let mut multi_window = MultiWindow::new();
    let mut fd = egui_multiwin::egui::FontData::from_static(COMPUTER_MODERN_FONT);
    fd.tweak.y_offset_factor = 1.0 / 3.0;
    multi_window.add_font("computermodern".to_string(), fd);

    for l in crate::library::LibraryHolder::get_user_libraries(&ac.dirs) {
        if let Some(lib) = &l.library {
            ac.libraries.insert(lib.name.clone(), l);
        }
    }

    if ac.args.len() > 1 {
        match ac.args[1].as_str() {
            "schematic" => {
                let _e =
                    multi_window.add(window::schematic::SchematicWindow::request(), &mut ac, &event_loop);
            }
            "library" => {
                let _e = multi_window.add(window::library::Library::request(), &mut ac, &event_loop);
            }
            _ => {
                let _e = multi_window.add(window::library::Library::request(), &mut ac, &event_loop);
            }
        }
    } else {
        let _e = multi_window.add(window::schematic::SchematicWindow::request(), &mut ac, &event_loop);
    }
    multi_window.run(event_loop, ac);
}

/// The central storage structure for the entire application
pub struct MyApp {
    /// The current electronics schematic open for the program. This may become a Vec<SchematicHolder> in the future.
    schematic: Option<SchematicHolder>,
    /// The libraries for the current setup
    libraries: HashMap<String, library::LibraryHolder>,
    /// The undo log for all libraries
    library_log: undo::Record<crate::library::LibraryAction>,
    /// The directories for the system
    dirs: Option<directories::ProjectDirs>,
    /// The command line arguments to the program
    args: Vec<String>,
    /// The units to display and modify for everything
    units: general::DisplayMode,
}

impl MyApp {
    fn process_event(&mut self, event: ipc::IpcMessage) -> Vec<NewWindowRequest> {
        let mut windows_to_create = vec![];
        match event {
            ipc::IpcMessage::NewSchematic => {
                windows_to_create.push(window::schematic::SchematicWindow::request());
            }
            ipc::IpcMessage::NewLibrary => {
                windows_to_create.push(window::library::Library::request());
            }
        }
        windows_to_create
    }
}

#[cfg(not(target_arch = "wasm32"))]
///Run an asynchronous object on a new thread. Maybe not the best way of accomplishing this, but it does work.
fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
///Run an asynchronous object on a new thread. Maybe not the best way of accomplishing this, but it does work.
fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
