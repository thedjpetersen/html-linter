pub(crate) mod index;
pub(crate) mod utils;

use markup5ever_rcdom::Handle;
use string_interner::Symbol;

pub(crate) use self::index::*;

#[derive(Debug)]
pub(crate) struct IndexedNode {
    pub tag_name: string_interner::DefaultSymbol,
    pub attributes: Vec<IndexedAttribute>,
    pub classes: Vec<string_interner::DefaultSymbol>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub source_info: SourceInfo,
    pub text_content: Option<string_interner::DefaultSymbol>,
    pub handle: Option<Handle>,
}

impl Default for IndexedNode {
    fn default() -> Self {
        Self {
            tag_name: string_interner::DefaultSymbol::try_from_usize(0).unwrap(),
            attributes: Vec::new(),
            classes: Vec::new(),
            parent: None,
            children: Vec::new(),
            source_info: SourceInfo {
                line: 0,
                column: 0,
                source: String::new(),
            },
            text_content: None,
            handle: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IndexedAttribute {
    pub name: string_interner::DefaultSymbol,
    pub value: string_interner::DefaultSymbol,
    pub quotes_type: QuotesType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum QuotesType {
    Single,
    Double,
    None,
}

#[derive(Clone, Debug)]
pub(crate) struct SourceInfo {
    pub line: usize,
    pub column: usize,
    pub source: String,
}

pub(crate) struct SourceMap {
    pub lines: Vec<String>,
    pub line_offsets: Vec<usize>,
}

impl SourceMap {
    pub fn new(source: &str) -> Self {
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut line_offsets = Vec::with_capacity(lines.len());
        let mut offset = 0;

        for line in &lines {
            line_offsets.push(offset);
            offset += line.len() + 1; // +1 for newline
        }

        Self {
            lines,
            line_offsets,
        }
    }

    pub fn get_position(&self, offset: usize) -> (usize, usize) {
        match self.line_offsets.binary_search(&offset) {
            Ok(line) => (line + 1, 1),
            Err(line) => {
                let line = if line == 0 { 0 } else { line - 1 };
                let column = offset - self.line_offsets[line] + 1;
                (line + 1, column)
            }
        }
    }
}
