//! Language definitions and comment queries for the comment remover.
//!
//! This module is the central registry of all programming languages supported
//! by the application. It provides:
//!
//! - An enum [`TreeSitterLanguage`] listing every language that can be
//!   processed, with each variant gated by a corresponding Cargo feature.
//! - Functions to obtain a tree‑sitter [`Language`] object for a given variant.
//! - Automatic language detection from file extensions.
//! - Parsing of language names from command‑line arguments (including aliases
//!   like `"rs"` for Rust).
//! - A static map [`COMMENT_QUERIES`] that stores the tree‑sitter query string
//!   used to locate comments in each language.
//!
//! # Feature Gating
//!
//! To keep the binary size small and to avoid pulling in unnecessary
//! tree‑sitter grammars, each language is protected by a Cargo feature.
//! For example, the `rust-lang` feature enables the Rust language and its
//! grammar. The enum variant `TreeSitterLanguage::Rust` is only present when
//! that feature is active. This means that code using this module must
//! conditionally compile matches or handle the possibility that a variant
//! may not exist.
//!
//! # Adding a New Language
//!
//! To add support for a new language:
//!
//! 1. Add a new feature to `Cargo.toml` (if it doesn't already exist).
//! 2. Add the corresponding tree‑sitter grammar crate as an optional
//!    dependency, gated by that feature.
//! 3. Add a new variant to the [`TreeSitterLanguage`] enum, gated with
//!    `#[cfg(feature = "your-language")]`.
//! 4. Implement the `get_language` method for the new variant, returning
//!    the grammar's `LANGUAGE` object.
//! 5. Add extension mappings in `detect_from_path`.
//! 6. Add aliases in `from_str` if desired.
//! 7. Insert the comment query into the `COMMENT_QUERIES` static map.
//! 8. Update the `supported()` list.
//! 9. Add tests for detection and parsing.
//!
//! # Example
//!
//! ```
//! use comment_remover::core::language::TreeSitterLanguage;
//! use std::path::Path;
//!
//! // Detect language from a file name
//! if let Some(lang) = TreeSitterLanguage::detect_from_path(Path::new("main.rs")) {
//!     println!("Detected language: {:?}", lang);
//! }
//!
//! // Parse a language name from a string
//! match TreeSitterLanguage::from_str("c++") {
//!     Ok(lang) => println!("Parsed language: {:?}", lang),
//!     Err(e) => eprintln!("{}", e),
//! }
//! ```

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use strum_macros::EnumString;
use tree_sitter::Language;

/// All programming languages that can have comments removed.
///
/// Each variant corresponds to a specific language and is conditionally
/// compiled based on the associated Cargo feature. Variants that are not
/// enabled in the current build simply do not exist, so pattern matches
/// must be guarded with `#[cfg(feature = "...")]` or be exhaustive only
/// over the enabled set.
///
/// The enum uses `strum`’s `EnumString` derive to allow case‑insensitive
/// parsing of the variant names (e.g., `"rust"`, `"RUST"`, `"Rust"` all
/// map to `TreeSitterLanguage::Rust`). Additional aliases are handled
/// manually in [`TreeSitterLanguage::from_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum TreeSitterLanguage {
    /// Bash / shell scripts (`.sh`, `.bash`). Feature: `bash`
    #[cfg(feature = "bash")]
    Bash,

    /// C (`.c`, `.h`). Feature: `c`
    #[cfg(feature = "c")]
    C,

    /// C# (`.cs`). Feature: `c-sharp`
    #[cfg(feature = "c-sharp")]
    CSharp,

    /// C++ (`.cpp`, `.cc`, `.cxx`, `.hpp`, …). Feature: `cpp`
    #[cfg(feature = "cpp")]
    Cpp,

    /// CSS (`.css`). Feature: `css`
    #[cfg(feature = "css")]
    Css,

    /// Go (`.go`). Feature: `go`
    #[cfg(feature = "go")]
    Go,

    /// Haskell (`.hs`). Feature: `haskell`
    #[cfg(feature = "haskell")]
    Haskell,

    /// HTML (`.html`, `.htm`). Feature: `html`
    #[cfg(feature = "html")]
    Html,

    /// Java (`.java`). Feature: `java`
    #[cfg(feature = "java")]
    Java,

    /// JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`). Feature: `javascript`
    #[cfg(feature = "javascript")]
    JavaScript,

    /// Lua (`.lua`). Feature: `lua`
    #[cfg(feature = "lua")]
    Lua,

    /// PHP (`.php`). Feature: `php`
    #[cfg(feature = "php")]
    Php,

    /// Python (`.py`, `.pyw`). Feature: `python`
    #[cfg(feature = "python")]
    Python,

    /// Ruby (`.rb`). Feature: `ruby`
    #[cfg(feature = "ruby")]
    Ruby,

    /// Rust (`.rs`). Feature: `rust-lang`
    #[cfg(feature = "rust-lang")]
    Rust,

    /// Scala (`.scala`). Feature: `scala`
    #[cfg(feature = "scala")]
    Scala,

    /// Swift (`.swift`). Feature: `swift`
    #[cfg(feature = "swift")]
    Swift,

    /// TypeScript (`.ts`, `.tsx`, `.mts`, `.cts`). Feature: `typescript`
    #[cfg(feature = "typescript")]
    TypeScript,

    /// SQL (`.sql`). Feature: `sql`
    #[cfg(feature = "sql")]
    Sql,

    /// Perl (`.pl`, `.pm`, `.t`). Feature: `perl`
    #[cfg(feature = "perl")]
    Perl,

    /// R language (`.r`, `.Rdata`). Feature: `r`
    #[cfg(feature = "r")]
    R,

    /// Dart (`.dart`). Feature: `dart`
    #[cfg(feature = "dart")]
    Dart,

    /// Elixir (`.ex`, `.exs`). Feature: `elixir`
    #[cfg(feature = "elixir")]
    Elixir,

    /// TOML (`.toml`). Feature: `toml`
    #[cfg(feature = "toml")]
    Toml,

    /// INI configuration (`.ini`, `.cfg`, `.conf`). Feature: `ini`
    #[cfg(feature = "ini")]
    Ini,
}

impl TreeSitterLanguage {
    /// Returns the tree‑sitter `Language` object associated with this language.
    ///
    /// This function is used to obtain a compiled grammar that can parse source
    /// code of the given language. It panics if the language variant exists but
    /// its feature is not enabled – however, because the variant itself is
    /// conditionally compiled, that situation cannot occur in practice.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::core::language::TreeSitterLanguage;
    /// # #[cfg(feature = "rust-lang")]
    /// # {
    /// let lang = TreeSitterLanguage::Rust.get_language();
    /// // `lang` can now be used to create a parser.
    /// # }
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
            // The `_` arm is unreachable because all variants are covered by the
            // features that are enabled; but it's required for exhaustiveness.
            _ => panic!("TreeSitterLanguage variant is not enabled in this build"),
        }
    }

    /// Attempts to detect the language from a file's extension.
    ///
    /// The file extension is extracted, converted to lowercase, and compared
    /// against a table of known mappings. If a match is found and the
    /// corresponding language feature is enabled, `Some(language)` is returned;
    /// otherwise `None`.
    ///
    /// This function is used when the user does not explicitly specify a
    /// language and the input comes from a file.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::core::language::TreeSitterLanguage;
    /// use std::path::Path;
    ///
    /// let path = Path::new("script.py");
    /// #[cfg(feature = "python")]
    /// assert_eq!(TreeSitterLanguage::detect_from_path(path), Some(TreeSitterLanguage::Python));
    /// ```
    pub fn detect_from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            // Existing languages
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

            // New languages
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

    /// Returns a list of language names that are enabled in the current build.
    ///
    /// This is used for generating help text or error messages that list
    /// available languages. The strings are the canonical names as they would
    /// be accepted by `from_str` (lowercase, no aliases).
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::core::language::TreeSitterLanguage;
    /// let supported = TreeSitterLanguage::supported();
    /// println!("Enabled languages: {:?}", supported);
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

    /// Parses a language name from a string, supporting common aliases.
    ///
    /// This function first checks for well‑known aliases (like `"rs"` for Rust,
    /// `"py"` for Python, etc.). If no alias matches, it falls back to
    /// `strum`’s case‑insensitive parse of the enum variant names.
    ///
    /// # Errors
    ///
    /// Returns an error string if the input does not correspond to any known
    /// language, or if the language is not enabled in the current build.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::core::language::TreeSitterLanguage;
    /// # #[cfg(feature = "rust-lang")] {
    /// assert_eq!(TreeSitterLanguage::from_str("rust").unwrap(), TreeSitterLanguage::Rust);
    /// assert_eq!(TreeSitterLanguage::from_str("rs").unwrap(), TreeSitterLanguage::Rust);
    /// # }
    /// ```
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            // Aliases for existing languages
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

            // Aliases for new languages
            #[cfg(feature = "perl")]
            "pl" => return Ok(Self::Perl),
            #[cfg(feature = "elixir")]
            "ex" => return Ok(Self::Elixir),
            _ => {}
        }

        // Fall back to strum's case-insensitive parse
        s.parse::<Self>().map_err(|_| {
            format!(
                "Language '{}' is not supported or not compiled in this build",
                s
            )
        })
    }
}

/// A static map from each supported language to its tree‑sitter query string.
///
/// The query string, when executed on a syntax tree, captures all nodes that
/// represent comments. The capture name is always `@comment`. The map is
/// populated at compile time based on enabled features.
///
/// # Example
///
/// ```
/// # use comment_remover::core::language::{COMMENT_QUERIES, TreeSitterLanguage};
/// # #[cfg(feature = "rust-lang")] {
/// if let Some(query) = COMMENT_QUERIES.get(&TreeSitterLanguage::Rust) {
///     println!("Rust comment query: {}", query);
/// }
/// # }
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