pub mod c_type;
pub mod shell_type;
pub mod xml_type;

pub use c_type::remove_c_type_comments;
pub use shell_type::{remove_hash_comments_basic, remove_python_comments, remove_shell_comments};
pub use xml_type::xml_type_remover;
