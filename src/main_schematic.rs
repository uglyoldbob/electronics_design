//! This package is software for designing electronic schematics and associated circuit boards.

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

mod main_common;
use main_common::*;

mod ipc;

fn main() {
    let instance = single_instance::SingleInstance::new(PACKAGE_NAME).unwrap();

    let mut ipcname = String::new();
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let dirs = DIRS.clone();
        let prefix = if let Some(dirs) = &dirs {
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
        bincode::serialize_into(ipc_sender, &ipc::IpcMessage::NewSchematic).unwrap();
    } else {
        drop(instance);
        /// The name of the main executable
        const NAME: &str = if cfg!(target_os = "windows") {
            "./electronics_design.exe"
        } else {
            "./electronics_design"
        };
        std::process::Command::new(NAME)
            .arg("schematic")
            .spawn()
            .expect("Failed to run main program");
    }
}
