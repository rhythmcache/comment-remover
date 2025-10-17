pub mod c_type;
pub mod go;
pub mod js;
pub mod rust;
pub mod shell_type;
pub mod xml_type;

pub use c_type::remove_c_type_comments;
pub use go::remove_go_comments;
pub use js::remove_js_comments;
pub use rust::remove_rust_comments;
pub use shell_type::{remove_hash_comments_basic, remove_python_comments, remove_shell_comments};
pub use xml_type::xml_type_remover;
