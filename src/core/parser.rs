use std::cell::RefCell;
use tree_sitter::Parser as TSParser;
use tree_sitter::Tree;

use crate::core::language::TreeSitterLanguage;
use crate::error::{AppError, Result};

thread_local! {
    static PARSER_CACHE: RefCell<Option<(TreeSitterLanguage, TSParser)>> =
        const { RefCell::new(None) };
}

pub fn parse(input: &str, language: TreeSitterLanguage) -> Result<Tree> {
    PARSER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        let parser = if let Some((cached_lang, parser)) = cache.as_mut() {
            if *cached_lang == language {
                parser
            } else {
                let mut new_parser = TSParser::new();
                new_parser
                    .set_language(&language.get_language())
                    .map_err(|e| AppError::TreeSitter(format!("Failed to load grammar: {}", e)))?;
                *cache = Some((language, new_parser));
                &mut cache.as_mut().unwrap().1
            }
        } else {
            let mut new_parser = TSParser::new();
            new_parser
                .set_language(&language.get_language())
                .map_err(|e| AppError::TreeSitter(format!("Failed to load grammar: {}", e)))?;
            let _ = cache.insert((language, new_parser));
            &mut cache.as_mut().unwrap().1
        };

        parser
            .parse(input, None)
            .ok_or_else(|| AppError::Parse("Failed to parse input".to_string()))
    })
}

pub fn clear_cache() {
    PARSER_CACHE.with(|cache| {
        *cache.borrow_mut() = None;
    });
}
