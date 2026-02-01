pub mod go;
pub mod python;
pub mod typescript;

pub use go::GoParser;
pub use python::PythonParser;
pub use typescript::{Language, TypeScriptParser};
