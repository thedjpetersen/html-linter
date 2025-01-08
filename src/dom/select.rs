use parking_lot::RwLock;
use std::collections::HashMap;
use string_interner::{DefaultSymbol, StringInterner};

// Move selector-related structs
#[derive(Clone, Debug)]
pub enum AttributeSelector {
    Exists(DefaultSymbol),                    // [attr]
    Equals(DefaultSymbol, DefaultSymbol),     // [attr=value]
    StartsWith(DefaultSymbol, DefaultSymbol), // [attr^=value]
    EndsWith(DefaultSymbol, DefaultSymbol),   // [attr$=value]
    Contains(DefaultSymbol, DefaultSymbol),   // [attr*=value]
}

#[derive(Clone, Debug)]
pub struct Selector {
    pub(crate) alternatives: Vec<SelectorPart>,
}

#[derive(Clone, Debug)]
pub struct SelectorPart {
    pub(crate) element: Option<DefaultSymbol>,
    pub(crate) classes: Vec<DefaultSymbol>,
    pub(crate) id: Option<DefaultSymbol>,
    pub(crate) attributes: Vec<AttributeSelector>,
}

pub struct SelectorEngine {
    selector_cache: RwLock<HashMap<String, Selector>>,
}

impl SelectorEngine {
    pub fn new() -> Self {
        Self {
            selector_cache: RwLock::new(HashMap::with_capacity(64)),
        }
    }

    pub fn parse_selector(&self, selector: &str, interner: &RwLock<StringInterner>) -> Selector {
        // Handle universal selector "*" explicitly
        if selector == "*" {
            return Selector {
                alternatives: vec![SelectorPart {
                    element: None,
                    classes: Vec::new(),
                    id: None,
                    attributes: Vec::new(),
                }],
            };
        }

        let mut alternatives = Vec::new();

        // Split by commas and handle each part
        for part in selector.split(',') {
            let part = part.trim(); // Handle potential spaces after commas
            if part.is_empty() {
                continue;
            }

            let mut element = None;
            let mut classes = Vec::with_capacity(4);
            let mut id = None;
            let mut attributes = Vec::new();
            let mut token = String::with_capacity(32);
            let mut chars = part.chars().peekable();

            while let Some(c) = chars.next() {
                match c {
                    '[' => {
                        if !token.is_empty() {
                            element = Some(interner.write().get_or_intern(&token));
                            token.clear();
                        }

                        // Parse attribute name
                        while let Some(&c) = chars.peek() {
                            if c == '=' || c == '^' || c == '$' || c == '*' || c == ']' {
                                break;
                            }
                            token.push(chars.next().unwrap());
                        }
                        let attr_name = interner.write().get_or_intern(&token);
                        token.clear();

                        // Parse operator and value if present
                        match chars.next() {
                            Some(']') => {
                                attributes.push(AttributeSelector::Exists(attr_name));
                            }
                            Some('=') => {
                                // Skip quote if present
                                if let Some(&'"') | Some(&'\'') = chars.peek() {
                                    chars.next();
                                }

                                while let Some(&c) = chars.peek() {
                                    if c == '"' || c == '\'' || c == ']' {
                                        break;
                                    }
                                    token.push(chars.next().unwrap());
                                }

                                // Skip closing quote if present
                                if let Some(&'"') | Some(&'\'') = chars.peek() {
                                    chars.next();
                                }

                                let value = interner.write().get_or_intern(&token);
                                attributes.push(AttributeSelector::Equals(attr_name, value));
                                token.clear();

                                // Skip closing bracket
                                chars.next();
                            }
                            Some(c) => {
                                match c {
                                    '^' | '$' | '*' => {
                                        let op = c;
                                        // Skip = character
                                        chars.next();

                                        // Skip quote if present
                                        if let Some(&'"') | Some(&'\'') = chars.peek() {
                                            chars.next();
                                        }

                                        while let Some(&c) = chars.peek() {
                                            if c == '"' || c == '\'' || c == ']' {
                                                break;
                                            }
                                            token.push(chars.next().unwrap());
                                        }

                                        // Skip closing quote if present
                                        if let Some(&'"') | Some(&'\'') = chars.peek() {
                                            chars.next();
                                        }

                                        let value = interner.write().get_or_intern(&token);
                                        let selector = match op {
                                            '^' => AttributeSelector::StartsWith(attr_name, value),
                                            '$' => AttributeSelector::EndsWith(attr_name, value),
                                            '*' => AttributeSelector::Contains(attr_name, value),
                                            _ => unreachable!(),
                                        };
                                        attributes.push(selector);
                                        token.clear();

                                        // Skip closing bracket
                                        chars.next();
                                    }
                                    _ => {}
                                }
                            }
                            None => break,
                        }
                    }
                    '#' => {
                        if !token.is_empty() {
                            element = Some(interner.write().get_or_intern(&token));
                            token.clear();
                        }
                        while let Some(&c) = chars.peek() {
                            if c == '.' || c == '#' {
                                break;
                            }
                            token.push(chars.next().unwrap());
                        }
                        id = Some(interner.write().get_or_intern(&token));
                        token.clear();
                    }
                    '.' => {
                        if !token.is_empty() {
                            element = Some(interner.write().get_or_intern(&token));
                            token.clear();
                        }
                        while let Some(&c) = chars.peek() {
                            if c == '.' || c == '#' {
                                break;
                            }
                            token.push(chars.next().unwrap());
                        }
                        classes.push(interner.write().get_or_intern(&token));
                        token.clear();
                    }
                    _ => token.push(c),
                }
            }

            if !token.is_empty() {
                element = Some(interner.write().get_or_intern(&token));
            }

            alternatives.push(SelectorPart {
                element,
                classes,
                id,
                attributes,
            });
        }

        Selector { alternatives }
    }

    pub fn get_or_parse_selector(
        &self,
        selector: &str,
        interner: &RwLock<StringInterner>,
    ) -> Selector {
        // Fast path: check cache first with read lock
        let cache = self.selector_cache.read();
        if let Some(sel) = cache.get(selector) {
            return sel.clone();
        }
        drop(cache);

        // Parse and cache the selector
        let sel = self.parse_selector(selector, interner);
        self.selector_cache
            .write()
            .insert(selector.to_string(), sel.clone());
        sel
    }
}
