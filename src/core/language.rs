//! Language support and detection for tree‑sitter based comment removal.
//!
//! This module defines the [`TreeSitterLanguage`] enum, which lists all
//! programming languages that can be processed. Each variant is conditionally
//! compiled based on the corresponding feature flag (e.g., `bash`, `python`).
//! The module also provides:
//!
//! * Language detection from file extensions ([`detect_from_path`]).
//! * Retrieval of the underlying tree‑sitter [`Language`] object ([`get_language`]).
//! * A list of supported language names ([`supported`]).
//! * String‑to‑language parsing with common aliases ([`from_str`]).
//! * A global static map [`COMMENT_QUERIES`] that holds the tree‑sitter query
//!   string for finding comments in each language.
//!
//! # Feature Flags
//!
//! Each language is gated by a Cargo feature. For example, to enable Rust
//! support you must compile with the `rust-lang` feature. This allows you to
//! control the binary size and only include the grammars you need.
//!
//! # Examples
//!
//! Detecting language from a file path:
//!
//! ```
//! use comment_remover::core::language::TreeSitterLanguage;
//! use std::path::Path;
//!
//! if let Some(lang) = TreeSitterLanguage::detect_from_path(Path::new("main.rs")) {
//!     println!("Detected language: {:?}", lang);
//! }
//! ```
//!
//! Getting a language from a string (e.g., from CLI):
//!
//! ```
//! use comment_remover::core::language::TreeSitterLanguage;
//!
//! let lang = TreeSitterLanguage::from_str("python").unwrap();
//! assert_eq!(lang, TreeSitterLanguage::Python);
//! ```
//!
//! Using the comment query for a language:
//!
//! ```
//! use comment_remover::core::language::{COMMENT_QUERIES, TreeSitterLanguage};
//!
//! let query = COMMENT_QUERIES.get(&TreeSitterLanguage::Rust).unwrap();
//! println!("Rust comment query: {}", query);
//! ```

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use strum_macros::EnumString;
use tree_sitter::Language;

/// Enumeration of all programming languages supported by the comment remover.
///
/// Each variant corresponds to a tree‑sitter grammar and is conditionally
/// compiled only when its feature is enabled. The enum is `Copy`, `Clone`,
/// and can be parsed from a string (case‑insensitive) via the `strum` derive.
///
/// # Variants
///
/// The list includes languages such as Rust, Python, JavaScript, etc.
/// Refer to the Cargo features for the exact set available in your build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum TreeSitterLanguage {
    /// Bash / Shell scripts (feature = "bash")
    #[cfg(feature = "bash")]
    Bash,

    /// C (feature = "c")
    #[cfg(feature = "c")]
    C,

    /// C# (feature = "c-sharp")
    #[cfg(feature = "c-sharp")]
    CSharp,

    /// C++ (feature = "cpp")
    #[cfg(feature = "cpp")]
    Cpp,

    /// CSS (feature = "css")
    #[cfg(feature = "css")]
    Css,

    /// Go (feature = "go")
    #[cfg(feature = "go")]
    Go,

    /// Haskell (feature = "haskell")
    #[cfg(feature = "haskell")]
    Haskell,

    /// HTML (feature = "html")
    #[cfg(feature = "html")]
    Html,

    /// Java (feature = "java")
    #[cfg(feature = "java")]
    Java,

    /// JavaScript (feature = "javascript")
    #[cfg(feature = "javascript")]
    JavaScript,

    /// Lua (feature = "lua")
    #[cfg(feature = "lua")]
    Lua,

    /// PHP (feature = "php")
    #[cfg(feature = "php")]
    Php,

    /// Python (feature = "python")
    #[cfg(feature = "python")]
    Python,

    /// Ruby (feature = "ruby")
    #[cfg(feature = "ruby")]
    Ruby,

    /// Rust (feature = "rust-lang")
    #[cfg(feature = "rust-lang")]
    Rust,

    /// Scala (feature = "scala")
    #[cfg(feature = "scala")]
    Scala,

    /// Swift (feature = "swift")
    #[cfg(feature = "swift")]
    Swift,

    /// TypeScript (feature = "typescript")
    #[cfg(feature = "typescript")]
    TypeScript,

    /// SQL (feature = "sql")
    #[cfg(feature = "sql")]
    Sql,

    /// Perl (feature = "perl")
    #[cfg(feature = "perl")]
    Perl,

    /// R (feature = "r")
    #[cfg(feature = "r")]
    R,

    /// Dart (feature = "dart")
    #[cfg(feature = "dart")]
    Dart,

    /// Elixir (feature = "elixir")
    #[cfg(feature = "elixir")]
    Elixir,

    /// TOML (feature = "toml")
    #[cfg(feature = "toml")]
    Toml,

    /// INI / configuration files (feature = "ini")
    #[cfg(feature = "ini")]
    Ini,
}

impl TreeSitterLanguage {
    /// Returns the underlying tree‑sitter `Language` object for this variant.
    ///
    /// This is used to configure a parser or to create queries. The method is
    /// safe to call only when the corresponding feature is enabled; otherwise
    /// the variant would not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use comment_remover::core::language::TreeSitterLanguage;
    /// use tree_sitter::Parser;
    ///
    /// let mut parser = Parser::new();
    /// parser.set_language(&TreeSitterLanguage::Python.get_language()).unwrap();
    /// ```
    pub fn get_language(&self) -> Language {
        match self {
            #[cfg(feature = "bash")]
            Self::Bash => tree_sitter_bash::LANGUAGE.into(),
            #[cfg(feature = "c")]
            Self::C => tree_sitter_c::LANGUAGE.into(),
            #[cfg(feature = "c-sharp")]
            Self::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            #[cfg(feature = "cpp")]
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            #[cfg(feature = "css")]
            Self::Css => tree_sitter_css::LANGUAGE.into(),
            #[cfg(feature = "go")]
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            #[cfg(feature = "haskell")]
            Self::Haskell => tree_sitter_haskell::LANGUAGE.into(),
            #[cfg(feature = "html")]
            Self::Html => tree_sitter_html::LANGUAGE.into(),
            #[cfg(feature = "java")]
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            #[cfg(feature = "javascript")]
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            #[cfg(feature = "lua")]
            Self::Lua => tree_sitter_lua::LANGUAGE.into(),
            #[cfg(feature = "php")]
            Self::Php => tree_sitter_php::LANGUAGE_PHP.into(),
            #[cfg(feature = "python")]
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            #[cfg(feature = "ruby")]
            Self::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            #[cfg(feature = "rust-lang")]
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            #[cfg(feature = "scala")]
            Self::Scala => tree_sitter_scala::LANGUAGE.into(),
            #[cfg(feature = "swift")]
            Self::Swift => tree_sitter_swift::LANGUAGE.into(),
            #[cfg(feature = "typescript")]
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),

            #[cfg(feature = "sql")]
            Self::Sql => tree_sitter_sql::LANGUAGE.into(),
            #[cfg(feature = "perl")]
            Self::Perl => tree_sitter_perl::LANGUAGE.into(),
            #[cfg(feature = "r")]
            Self::R => tree_sitter_r::LANGUAGE.into(),
            #[cfg(feature = "dart")]
            Self::Dart => tree_sitter_dart::LANGUAGE.into(),
            #[cfg(feature = "elixir")]
            Self::Elixir => tree_sitter_elixir::LANGUAGE.into(),
            #[cfg(feature = "toml")]
            Self::Toml => tree_sitter_toml::LANGUAGE.into(),
            #[cfg(feature = "ini")]
            Self::Ini => tree_sitter_ini::LANGUAGE.into(),
        }
    }

    /// Attempts to detect the programming language from a file path by its extension.
    ///
    /// The mapping is based on common file extensions (e.g., `.rs` → Rust,
    /// `.py` → Python). If the extension is recognised and the corresponding
    /// feature is enabled, `Some(lang)` is returned; otherwise `None`.
    ///
    /// # Arguments
    ///
    /// * `path` - A file path whose extension will be examined.
    ///
    /// # Returns
    ///
    /// * `Some(TreeSitterLanguage)` if a matching, enabled language is found.
    /// * `None` if the extension is unknown or the language feature is disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use comment_remover::core::language::TreeSitterLanguage;
    /// use std::path::Path;
    ///
    /// let path = Path::new("script.py");
    /// assert_eq!(TreeSitterLanguage::detect_from_path(path), Some(TreeSitterLanguage::Python));
    /// ```
    pub fn detect_from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            #[cfg(feature = "bash")]
            "sh" | "bash" => Some(Self::Bash),
            #[cfg(feature = "c")]
            "c" | "h" => Some(Self::C),
            #[cfg(feature = "c-sharp")]
            "cs" => Some(Self::CSharp),
            #[cfg(feature = "cpp")]
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "c++" => Some(Self::Cpp),
            #[cfg(feature = "css")]
            "css" => Some(Self::Css),
            #[cfg(feature = "go")]
            "go" => Some(Self::Go),
            #[cfg(feature = "haskell")]
            "hs" => Some(Self::Haskell),
            #[cfg(feature = "html")]
            "html" | "htm" => Some(Self::Html),
            #[cfg(feature = "java")]
            "java" => Some(Self::Java),
            #[cfg(feature = "javascript")]
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            #[cfg(feature = "lua")]
            "lua" => Some(Self::Lua),
            #[cfg(feature = "php")]
            "php" => Some(Self::Php),
            #[cfg(feature = "python")]
            "py" | "pyw" => Some(Self::Python),
            #[cfg(feature = "ruby")]
            "rb" => Some(Self::Ruby),
            #[cfg(feature = "rust-lang")]
            "rs" => Some(Self::Rust),
            #[cfg(feature = "scala")]
            "scala" => Some(Self::Scala),
            #[cfg(feature = "swift")]
            "swift" => Some(Self::Swift),
            #[cfg(feature = "typescript")]
            "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),

            #[cfg(feature = "sql")]
            "sql" => Some(Self::Sql),
            #[cfg(feature = "perl")]
            "pl" | "pm" | "t" => Some(Self::Perl),
            #[cfg(feature = "r")]
            "r" | "rdata" => Some(Self::R),
            #[cfg(feature = "dart")]
            "dart" => Some(Self::Dart),
            #[cfg(feature = "elixir")]
            "ex" | "exs" => Some(Self::Elixir),
            #[cfg(feature = "toml")]
            "toml" => Some(Self::Toml),
            #[cfg(feature = "ini")]
            "ini" | "cfg" | "conf" => Some(Self::Ini),
            _ => None,
        }
    }

    /// Returns a list of all language names that are supported in the current build.
    ///
    /// The list is generated based on enabled features and includes both the
    /// standard names (e.g., `"python"`) and common aliases. This is useful
    /// for generating help text or error messages.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the names of all enabled languages.
    ///
    /// # Examples
    ///
    /// ```
    /// use comment_remover::core::language::TreeSitterLanguage;
    ///
    /// let supported = TreeSitterLanguage::supported();
    /// println!("Supported languages: {}", supported.join(", "));
    /// ```
    pub fn supported() -> Vec<&'static str> {
        let mut v = Vec::new();
        #[cfg(feature = "bash")]
        v.push("bash");
        #[cfg(feature = "c")]
        v.push("c");
        #[cfg(feature = "c-sharp")]
        v.push("c#");
        #[cfg(feature = "cpp")]
        v.push("c++");
        #[cfg(feature = "css")]
        v.push("css");
        #[cfg(feature = "go")]
        v.push("go");
        #[cfg(feature = "haskell")]
        v.push("haskell");
        #[cfg(feature = "html")]
        v.push("html");
        #[cfg(feature = "java")]
        v.push("java");
        #[cfg(feature = "javascript")]
        v.push("javascript");
        #[cfg(feature = "lua")]
        v.push("lua");
        #[cfg(feature = "php")]
        v.push("php");
        #[cfg(feature = "python")]
        v.push("python");
        #[cfg(feature = "ruby")]
        v.push("ruby");
        #[cfg(feature = "rust-lang")]
        v.push("rust");
        #[cfg(feature = "scala")]
        v.push("scala");
        #[cfg(feature = "swift")]
        v.push("swift");
        #[cfg(feature = "typescript")]
        v.push("typescript");
        #[cfg(feature = "sql")]
        v.push("sql");
        #[cfg(feature = "perl")]
        v.push("perl");
        #[cfg(feature = "r")]
        v.push("r");
        #[cfg(feature = "dart")]
        v.push("dart");
        #[cfg(feature = "elixir")]
        v.push("elixir");
        #[cfg(feature = "toml")]
        v.push("toml");
        #[cfg(feature = "ini")]
        v.push("ini");
        v
    }

    /// Parses a string into a `TreeSitterLanguage`, accepting common aliases.
    ///
    /// This function first checks for common shorthand aliases (e.g., `"py"`
    /// for Python) and then falls back to the case‑insensitive `strum` parse.
    /// It returns a `Result` where the error contains a descriptive message.
    ///
    /// # Arguments
    ///
    /// * `s` - The language name or alias (e.g., `"rust"`, `"rs"`, `"c++"`).
    ///
    /// # Returns
    ///
    /// * `Ok(TreeSitterLanguage)` if the string matches a known, enabled language.
    /// * `Err(String)` with an error message if the language is unknown or disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use comment_remover::core::language::TreeSitterLanguage;
    ///
    /// assert!(TreeSitterLanguage::from_str("rust").is_ok());
    /// assert!(TreeSitterLanguage::from_str("rs").is_ok());
    /// assert!(TreeSitterLanguage::from_str("c++").is_ok());
    /// assert!(TreeSitterLanguage::from_str("unknown").is_err());
    /// ```
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            #[cfg(feature = "bash")]
            "sh" => return Ok(Self::Bash),
            #[cfg(feature = "c-sharp")]
            "c#" | "csharp" => return Ok(Self::CSharp),
            #[cfg(feature = "cpp")]
            "c++" | "cc" | "cxx" => return Ok(Self::Cpp),
            #[cfg(feature = "go")]
            "golang" => return Ok(Self::Go),
            #[cfg(feature = "haskell")]
            "hs" => return Ok(Self::Haskell),
            #[cfg(feature = "javascript")]
            "js" => return Ok(Self::JavaScript),
            #[cfg(feature = "python")]
            "py" => return Ok(Self::Python),
            #[cfg(feature = "ruby")]
            "rb" => return Ok(Self::Ruby),
            #[cfg(feature = "rust-lang")]
            "rs" => return Ok(Self::Rust),
            #[cfg(feature = "typescript")]
            "ts" => return Ok(Self::TypeScript),

            #[cfg(feature = "perl")]
            "pl" => return Ok(Self::Perl),
            #[cfg(feature = "elixir")]
            "ex" => return Ok(Self::Elixir),
            _ => {}
        }

        s.parse::<Self>().map_err(|_| {
            format!(
                "Language '{}' is not supported or not compiled in this build",
                s
            )
        })
    }
}

/// A static map from language to the tree‑sitter query string that matches comments.
///
/// For each language, the query string captures all comment nodes (line comments,
/// block comments, etc.) under the capture name `@comment`. The queries are
/// used by [`CommentRemover`](crate::core::remover::CommentRemover) to locate
/// comment ranges.
///
/// The map is lazily initialized on first access.
///
/// # Examples
///
/// ```
/// use comment_remover::core::language::{COMMENT_QUERIES, TreeSitterLanguage};
///
/// let rust_query = COMMENT_QUERIES.get(&TreeSitterLanguage::Rust).unwrap();
/// assert!(rust_query.contains("line_comment") || rust_query.contains("comment"));
/// ```
pub static COMMENT_QUERIES: Lazy<HashMap<TreeSitterLanguage, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    #[cfg(feature = "bash")]
    m.insert(TreeSitterLanguage::Bash, "(comment) @comment");
    #[cfg(feature = "c")]
    m.insert(TreeSitterLanguage::C, "(comment) @comment");
    #[cfg(feature = "c-sharp")]
    m.insert(TreeSitterLanguage::CSharp, "(comment) @comment");
    #[cfg(feature = "cpp")]
    m.insert(TreeSitterLanguage::Cpp, "(comment) @comment");
    #[cfg(feature = "css")]
    m.insert(TreeSitterLanguage::Css, "(comment) @comment");
    #[cfg(feature = "go")]
    m.insert(TreeSitterLanguage::Go, "(comment) @comment");
    #[cfg(feature = "haskell")]
    m.insert(TreeSitterLanguage::Haskell, "(comment) @comment");
    #[cfg(feature = "html")]
    m.insert(TreeSitterLanguage::Html, "(comment) @comment");
    #[cfg(feature = "java")]
    m.insert(
        TreeSitterLanguage::Java,
        "(line_comment) @comment (block_comment) @comment",
    );
    #[cfg(feature = "javascript")]
    m.insert(TreeSitterLanguage::JavaScript, "(comment) @comment");
    #[cfg(feature = "lua")]
    m.insert(TreeSitterLanguage::Lua, "(comment) @comment");
    #[cfg(feature = "php")]
    m.insert(TreeSitterLanguage::Php, "(comment) @comment");
    #[cfg(feature = "python")]
    m.insert(TreeSitterLanguage::Python, "(comment) @comment");
    #[cfg(feature = "ruby")]
    m.insert(TreeSitterLanguage::Ruby, "(comment) @comment");
    #[cfg(feature = "rust-lang")]
    m.insert(
        TreeSitterLanguage::Rust,
        "(line_comment) @comment (block_comment) @comment",
    );
    #[cfg(feature = "scala")]
    m.insert(TreeSitterLanguage::Scala, "(comment) @comment");
    #[cfg(feature = "swift")]
    m.insert(TreeSitterLanguage::Swift, "(comment) @comment");
    #[cfg(feature = "typescript")]
    m.insert(TreeSitterLanguage::TypeScript, "(comment) @comment");

    #[cfg(feature = "sql")]
    m.insert(TreeSitterLanguage::Sql, "(comment) @comment");
    #[cfg(feature = "perl")]
    m.insert(TreeSitterLanguage::Perl, "(comment) @comment");
    #[cfg(feature = "r")]
    m.insert(TreeSitterLanguage::R, "(comment) @comment");
    #[cfg(feature = "dart")]
    m.insert(TreeSitterLanguage::Dart, "(comment) @comment");
    #[cfg(feature = "elixir")]
    m.insert(TreeSitterLanguage::Elixir, "(comment) @comment");
    #[cfg(feature = "toml")]
    m.insert(TreeSitterLanguage::Toml, "(comment) @comment");
    #[cfg(feature = "ini")]
    m.insert(TreeSitterLanguage::Ini, "(comment) @comment");

    m
});