use std::cell::RefCell;
use tree_sitter::{Parser as TSParser, Tree};

use crate::core::language::TreeSitterLanguage;
use crate::error::{AppError, Result};

thread_local! {
    static PARSER_CACHE: RefCell<Option<(TreeSitterLanguage, TSParser)>> = const { RefCell::new(None) };
}

fn create_parser(language: TreeSitterLanguage) -> Result<TSParser> {
    let mut parser = TSParser::new();
    parser.set_language(&language.get_language()).map_err(|e| {
        AppError::TreeSitter(format!("Failed to load grammar for {:?}: {}", language, e))
    })?;
    Ok(parser)
}

pub fn parse(input: &str, language: TreeSitterLanguage) -> Result<Tree> {
    PARSER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let parser = match cache.as_mut() {
            Some((lang, parser)) if *lang == language => parser,
            _ => {
                let parser = create_parser(language)?;
                *cache = Some((language, parser));
                &mut cache.as_mut().unwrap().1
            }
        };
        parser
            .parse(input, None)
            .ok_or_else(|| AppError::Parse("Failed to parse input".to_string()))
    })
}

pub fn clear_cache() {
    PARSER_CACHE.with(|cache| *cache.borrow_mut() = None);
}
