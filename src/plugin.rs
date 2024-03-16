use std::ffi::{c_char, CStr};
use std::path::Path;
use std::str::Utf8Error;

use sharedlib::{DataRc, LibRc, Symbol};


#[derive(Debug)]
pub struct Plugin {
    pub library: LibRc,
    // pub plugin_declaration: DataRc<PluginDeclaration>,
}
