use std::fs;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

use crate::core::language::{COMMENT_QUERIES, TreeSitterLanguage};
use crate::core::parser;
use crate::core::whitespace::collapse_whitespace;
use crate::error::{AppError, Result, io_error};

#[derive(Debug, Clone)]
pub struct CommentRemover {
    language: TreeSitterLanguage,
    collapse: Option<usize>,
}

impl CommentRemover {
    pub fn new(language: TreeSitterLanguage, collapse: Option<usize>) -> Self {
        Self { language, collapse }
    }

    pub fn process_str(&self, input: &str) -> Result<String> {
        let tree = parser::parse(input, self.language)?;
        let query_str = COMMENT_QUERIES
            .get(&self.language)
            .ok_or_else(|| AppError::UnsupportedLanguage(format!("{:?}", self.language)))?;

        let ts_lang = self.language.get_language();
        let query = Query::new(&ts_lang, query_str)
            .map_err(|e| AppError::Query(format!("Failed to create query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), input.as_bytes());

        let mut comment_ranges: Vec<std::ops::Range<usize>> = Vec::new();
        while let Some(m) = matches.next() {
            for capture in m.captures {
                comment_ranges.push(capture.node.byte_range());
            }
        }

        comment_ranges.sort_by_key(|r| r.start);

        let mut result = String::with_capacity(input.len());
        let mut last_pos = 0;
        for range in comment_ranges {
            result.push_str(&input[last_pos..range.start]);

            result.extend(input[range.clone()].chars().filter(|&c| c == '\n'));
            last_pos = range.end;
        }
        result.push_str(&input[last_pos..]);

        if let Some(max) = self.collapse {
            result = collapse_whitespace(&result, max);
        }

        Ok(result)
    }

    pub fn process_file(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path).map_err(|e| io_error(path, e))?;
        self.process_str(&content)
    }
}
