//! This package is software for designing electronic schematics and associated circuit boards.

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod general;
mod library;
mod schematic;
mod symbol;

use std::collections::HashMap;

use egui_multiwin::multi_window::NewWindowRequest;

use crate::schematic::SchematicHolder;

mod window;

/// The name of the overall software package
const PACKAGE_NAME: &str = "UglyOldBob Electronics";

fn main() {
    let instance = single_instance::SingleInstance::new(PACKAGE_NAME).unwrap();
    if !instance.is_single() {
        let ipc_sender =
            interprocess::local_socket::LocalSocketStream::connect(PACKAGE_NAME).unwrap();
        bincode::serialize_into(ipc_sender, &general::IpcMessage::NewProcess).unwrap();
        return;
    }

    let event_loop = egui_multiwin::glutin::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let (sender, receiver) = std::sync::mpsc::channel::<general::IpcMessage>();
    let proxy = proxy.clone();
    std::thread::spawn(move || {
        let proxy = proxy.clone();
        let ipc_listener =
            interprocess::local_socket::LocalSocketListener::bind(PACKAGE_NAME).unwrap();
        for i in ipc_listener.incoming() {
            if let Ok(mut i) = i {
                let sender = sender.clone();
                let proxy = proxy.clone();
                std::thread::spawn(move || {
                    let s = sender;
                    loop {
                        let msg = bincode::deserialize_from::<
                            &mut interprocess::local_socket::LocalSocketStream,
                            general::IpcMessage,
                        >(&mut i);
                        if let Ok(msg) = msg {
                            println!("Received a {:?}", msg);
                            if s.send(msg).is_err() {
                                break;
                            }
                            proxy.send_event(()).ok();
                        } else {
                            break;
                        }
                    }
                });
            }
        }
    });

    let mut multi_window = egui_multiwin::multi_window::MultiWindow::new();
    let root_window = window::schematic::SchematicWindow::request();
    let libedit = window::library::Library::request();

    let mut ac = MyApp {
        schematic: None,
        libraries: HashMap::new(),
        library_log: undo::Record::new(),
        dirs: directories::ProjectDirs::from("com", "UglyOldBob", "ElectronicsDesign"),
        receiver,
    };

    for l in crate::library::LibraryHolder::get_user_libraries(&ac.dirs) {
        ac.libraries.insert(l.library.name.clone(), Some(l));
    }
    let _e = multi_window.add(libedit, &event_loop);
    multi_window.run(event_loop, ac);
}

/// The central storage structure for the entire application
pub struct MyApp {
    /// The current electronics schematic open for the program. This may become a Vec<SchematicHolder> in the future.
    schematic: Option<SchematicHolder>,
    /// The libraries for the current setup
    libraries: HashMap<String, Option<library::LibraryHolder>>,
    /// The undo log for all libraries
    library_log: undo::Record<crate::library::LibraryAction>,
    /// The directories for the system
    dirs: Option<directories::ProjectDirs>,
    /// For receiving messages from other processes
    receiver: std::sync::mpsc::Receiver<general::IpcMessage>,
}

impl MyApp {
    /// Processes messages received from other processes
    pub fn receive_ipc(&mut self) -> Vec<NewWindowRequest<MyApp>> {
        let mut windows_to_create = vec![];
        while let Ok(m) = self.receiver.try_recv() {
            println!("gui received {:?}", m);
            match m {
                general::IpcMessage::NewProcess => {
                    windows_to_create.push(window::schematic::SchematicWindow::request());
                }
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
