use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use strum_macros::EnumString;
use tree_sitter::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum TreeSitterLanguage {
    #[cfg(feature = "bash")]
    Bash,

    #[cfg(feature = "c")]
    C,

    #[cfg(feature = "c-sharp")]
    CSharp,

    #[cfg(feature = "cpp")]
    Cpp,

    #[cfg(feature = "css")]
    Css,

    #[cfg(feature = "go")]
    Go,

    #[cfg(feature = "haskell")]
    Haskell,

    #[cfg(feature = "html")]
    Html,

    #[cfg(feature = "java")]
    Java,

    #[cfg(feature = "javascript")]
    JavaScript,

    #[cfg(feature = "lua")]
    Lua,

    #[cfg(feature = "php")]
    Php,

    #[cfg(feature = "python")]
    Python,

    #[cfg(feature = "ruby")]
    Ruby,

    #[cfg(feature = "rust-lang")]
    Rust,

    #[cfg(feature = "scala")]
    Scala,

    #[cfg(feature = "swift")]
    Swift,

    #[cfg(feature = "typescript")]
    TypeScript,

    #[cfg(feature = "sql")]
    Sql,

    #[cfg(feature = "perl")]
    Perl,

    #[cfg(feature = "r")]
    R,

    #[cfg(feature = "dart")]
    Dart,

    #[cfg(feature = "elixir")]
    Elixir,

    #[cfg(feature = "toml")]
    Toml,

    #[cfg(feature = "ini")]
    Ini,
}

impl TreeSitterLanguage {
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
