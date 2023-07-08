//! This module defines what a library is.

use std::collections::HashMap;

use egui_multiwin::egui;

use crate::symbol::SymbolDefinition;

/// The actions that can be done to a library
pub enum LibraryAction {
    /// Create a new empty library, lib should be None
    CreateNewLibrary {
        /// The name of the library
        name: String,
        /// The holder for when creation is undone
        lib: Option<LibraryHolder>,
    },
    /// Delete an existing library, old_lib should be None
    DeleteLibrary {
        /// The name of the library
        name: String,
        /// Hold the old library when required
        old_lib: Option<LibraryHolder>,
    },
    /// Move text on by a certain amount
    MoveText {
        /// The name of the library
        libname: String,
        /// The symbol name
        symname: String,
        /// The text number
        textnum: usize,
        /// The delta to move by
        delta: crate::general::Coordinates,
    },
    /// Create a new text
    CreateText {
        /// The name of the library
        libname: String,
        /// The symbol name
        symname: String,
        /// The new text
        text: crate::schematic::TextOnPage,
    },
    /// Edits the text of a text object
    EditText {
        /// The name of the library
        libname: String,
        /// The symbol name
        symname: String,
        /// The text number
        textnum: usize,
        /// The old text
        old: String,
        /// The new text
        new: String,
    },
    /// Change the text color of a text object
    ChangeTextColor {
        /// The name of the library
        libname: String,
        /// The symbol name
        symname: String,
        /// The text number
        textnum: usize,
        /// The old text
        old: crate::schematic::Colors,
        /// The new text
        new: crate::schematic::Colors,
    },
    /// Delete a symbol from the library, symbol must be None
    DeleteSymbol {
        /// The name of the library
        libname: String,
        /// The name of the symbol to Delete
        symname: String,
        /// The deleted object
        symbol: Option<SymbolDefinition>,
    },
    /// Add a new blank symbol to the library
    CreateSymbol {
        /// The name of the library
        libname: String,
        /// The name of the symbol to Delete
        symname: String,
    },
    /// Add a pin to a symbol in the library
    CreatePin {
        /// The name of the library
        libname: String,
        /// The name of the symbol to Delete
        symname: String,
        /// The pin to add, must be a Some variant
        pin: Option<crate::symbol::Pin>,
    },
}

impl undo::Action for LibraryAction {
    type Target = HashMap<String, Option<LibraryHolder>>;

    type Output = ();

    fn apply(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            LibraryAction::CreateNewLibrary { name, lib } => {
                if let Some(l) = lib.take() {
                    target.insert(name.clone(), Some(l));
                } else {
                    target.insert(name.clone(), Some(LibraryHolder::new(name.clone())));
                }
            }
            LibraryAction::DeleteLibrary { name, old_lib } => {
                target
                    .remove(name)
                    .and_then(|l| l.map(|l| old_lib.insert(l)));
            }
            LibraryAction::MoveText {
                libname,
                symname,
                textnum,
                delta,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts[*textnum].location += *delta;
                    }
                }
            }
            LibraryAction::CreateText {
                libname,
                symname,
                text,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts.push(text.clone());
                    }
                }
            }
            LibraryAction::EditText {
                libname,
                symname,
                textnum,
                old: _,
                new,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts[*textnum].text = new.clone();
                    }
                }
            }
            LibraryAction::ChangeTextColor {
                libname,
                symname,
                textnum,
                old: _,
                new,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts[*textnum].color = *new;
                    }
                }
            }
            LibraryAction::DeleteSymbol {
                libname,
                symname,
                symbol,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    *symbol = target.library.syms.remove(symname);
                }
            }
            LibraryAction::CreateSymbol { libname, symname } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    target
                        .library
                        .syms
                        .insert(symname.clone(), SymbolDefinition::new(symname.clone()));
                }
            }
            LibraryAction::CreatePin {
                libname,
                symname,
                pin,
            } => {
                if let Some(p) = pin.take() {
                    if let Some(Some(target)) = target.get_mut(libname) {
                        if let Some(sym) = target.library.syms.get_mut(symname) {
                            sym.pins.push(p);
                        }
                    }
                }
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            LibraryAction::CreateNewLibrary { name, lib } => {
                if let Some(l) = target.remove(name) {
                    *lib = l;
                }
            }
            LibraryAction::DeleteLibrary { name, old_lib } => {
                target.insert(name.clone(), old_lib.take());
            }
            LibraryAction::MoveText {
                libname,
                symname,
                textnum,
                delta,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts[*textnum].location -= *delta;
                    }
                }
            }
            LibraryAction::CreateText {
                libname,
                symname,
                text: _,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts.pop();
                    }
                }
            }
            LibraryAction::EditText {
                libname,
                symname,
                textnum,
                old,
                new: _,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts[*textnum].text = old.clone();
                    }
                }
            }
            LibraryAction::ChangeTextColor {
                libname,
                symname,
                textnum,
                old,
                new: _,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        sym.texts[*textnum].color = *old;
                    }
                }
            }
            LibraryAction::DeleteSymbol {
                libname,
                symname,
                symbol,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    let sym = symbol.take();
                    if let Some(s) = sym {
                        target.library.syms.insert(symname.clone(), s);
                    }
                }
            }
            LibraryAction::CreateSymbol { libname, symname } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    target.library.syms.remove(symname);
                }
            }
            LibraryAction::CreatePin {
                libname,
                symname,
                pin,
            } => {
                if let Some(Some(target)) = target.get_mut(libname) {
                    if let Some(sym) = target.library.syms.get_mut(symname) {
                        *pin = sym.pins.pop();
                    }
                }
            }
        }
    }

    fn merge(&mut self, other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        match self {
            LibraryAction::CreateNewLibrary { name: _, lib: _ } => undo::Merged::No(other),
            LibraryAction::DeleteLibrary {
                name: _,
                old_lib: _,
            } => undo::Merged::No(other),
            LibraryAction::MoveText {
                libname,
                symname,
                textnum,
                delta,
            } => {
                if let LibraryAction::MoveText {
                    libname: libname2,
                    symname: symname2,
                    textnum: tn2,
                    delta: delta2,
                } = other
                {
                    if *libname == libname2 && *symname == symname2 && *textnum == tn2 {
                        if (*delta + delta2).less_than_epsilon() {
                            undo::Merged::Annul
                        } else {
                            *delta += delta2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(LibraryAction::MoveText {
                            libname: libname2,
                            symname: symname2,
                            textnum: tn2,
                            delta: delta2,
                        })
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
            LibraryAction::CreateText {
                libname: _,
                symname: _,
                text: _,
            } => undo::Merged::No(other),
            LibraryAction::EditText {
                libname,
                symname,
                textnum,
                old,
                new,
            } => {
                if let LibraryAction::EditText {
                    libname: libname2,
                    symname: symname2,
                    textnum: textnum2,
                    old: old2,
                    new: new2,
                } = other
                {
                    if *libname == libname2 && *symname == symname2 && *textnum == textnum2 {
                        if *old == new2 {
                            undo::Merged::Annul
                        } else {
                            *new = new2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(LibraryAction::EditText {
                            libname: libname2,
                            symname: symname2,
                            textnum: textnum2,
                            old: old2,
                            new: new2,
                        })
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
            LibraryAction::ChangeTextColor {
                libname,
                symname,
                textnum,
                old,
                new,
            } => {
                if let LibraryAction::ChangeTextColor {
                    libname: libname2,
                    symname: symname2,
                    textnum: textnum2,
                    old: old2,
                    new: new2,
                } = other
                {
                    if *libname == libname2 && *symname == symname2 && *textnum == textnum2 {
                        if *old == new2 {
                            undo::Merged::Annul
                        } else {
                            *new = new2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(LibraryAction::ChangeTextColor {
                            libname: libname2,
                            symname: symname2,
                            textnum: textnum2,
                            old: old2,
                            new: new2,
                        })
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
            LibraryAction::DeleteSymbol {
                libname: _,
                symname: _,
                symbol: _,
            } => undo::Merged::No(other),
            LibraryAction::CreateSymbol {
                libname: _,
                symname: _,
            } => undo::Merged::No(other),
            LibraryAction::CreatePin {
                libname: _,
                symname: _,
                pin: _,
            } => undo::Merged::No(other),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// A library. It is a collection of symbols, footprints, and 3d models
pub struct Library {
    /// The name of the library. Must be unique for the system
    pub name: String,
    /// The symbols defined in the library
    pub syms: HashMap<String, crate::symbol::SymbolDefinition>,
}

impl Library {
    /// Create a new blank library
    fn new(name: String) -> Self {
        Self {
            name,
            syms: HashMap::new(),
        }
    }
}

/// Separates data to be stored from data that is not to be stored
pub struct LibraryHolder {
    /// The library, containing stored data
    pub library: Library,
    /// Where the library is stored
    pub path: Option<crate::general::StoragePath>,
    /// The file format to save the library in
    format: crate::general::StorageFormat,
}

impl LibraryHolder {
    /// Sets the save path for the library
    pub fn set_path(&mut self, p: crate::general::StoragePath) {
        self.path = Some(p);
    }

    /// Returns true when the save function can probably run properly
    pub fn can_save(&self) -> bool {
        self.path.is_some()
    }

    /// Saves the library to wherever it has been configured to be saved
    /// Will return Ok if the path is None
    pub fn save(&self) -> Result<(), crate::general::StorageSaveError> {
        if let Some(path) = &self.path {
            let mut writer = path.writer()?;
            self.format.save(&mut writer, &self.library)
        } else {
            Ok(())
        }
    }

    /// Create a new blank library holder, with a new library
    pub fn new(name: String) -> Self {
        Self {
            library: Library::new(name),
            path: None,
            format: crate::general::StorageFormat::default(),
        }
    }

    /// Gets all user libraries
    pub fn get_user_libraries(dirs: &Option<directories::ProjectDirs>) -> Vec<LibraryHolder> {
        let mut libs = Vec::new();
        if let Some(dirs) = dirs {
            let folder = dirs.data_dir();
            let _ = std::fs::create_dir_all(folder);
            if let Ok(p) = std::fs::read_dir(folder) {
                let mut newlibs = p
                    .filter_map(|res| res.ok())
                    // Map the directory entries to paths
                    .map(|dir_entry| dir_entry.path())
                    // Filter out all paths with extensions other than `csv`
                    .filter_map(|path| {
                        if path.extension().map_or(false, |ext| ext == "uol") {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .filter_map(|path| {
                        Some(crate::general::StoragePath::LocalFilesystem(
                            path.into_os_string().into_string().unwrap(),
                        ))
                    })
                    .filter_map(|path| {
                        let reader = path.reader();
                        match reader {
                            Ok(mut reader) => {
                                let format = crate::general::StorageFormat::default();
                                match format.load::<Library>(&mut reader) {
                                    Ok(lib) => Some(LibraryHolder {
                                        library: lib,
                                        path: Some(path),
                                        format,
                                    }),
                                    Err(e) => {
                                        println!(
                                            "ERROR Loading library {} {}",
                                            path.to_string(),
                                            e.to_string()
                                        );
                                        None
                                    }
                                }
                            }
                            Err(e) => {
                                println!(
                                    "ERROR Loading library {} {}",
                                    path.to_string(),
                                    e.to_string()
                                );
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                libs.append(&mut newlibs);
            }
        }
        libs
    }
}
