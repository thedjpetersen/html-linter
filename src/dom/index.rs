use parking_lot::RwLock;
use std::collections::HashMap;
use string_interner::DefaultSymbol;
use string_interner::StringInterner;

use crate::dom::{IndexedAttribute, IndexedNode, QuotesType, SourceInfo, SourceMap};
// Optimized arena with pre-allocated capacity
pub struct NodeArena {
    nodes: Vec<IndexedNode>,
}

impl NodeArena {
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(1024),
        }
    }

    #[inline]
    pub fn allocate(&mut self) -> &mut IndexedNode {
        let idx = self.nodes.len();
        self.nodes.push(IndexedNode::default());
        &mut self.nodes[idx]
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&IndexedNode> {
        self.nodes.get(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut IndexedNode> {
        self.nodes.get_mut(index)
    }
}

// Optimized to use DefaultStringInterner directly
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
    alternatives: Vec<SelectorPart>,
}

#[derive(Clone, Debug)]
pub struct SelectorPart {
    element: Option<DefaultSymbol>,
    classes: Vec<DefaultSymbol>,
    id: Option<DefaultSymbol>,
    attributes: Vec<AttributeSelector>,
}

pub struct DOMIndex {
    pub arena: NodeArena,
    elements: HashMap<DefaultSymbol, Vec<usize>>,
    ids: HashMap<DefaultSymbol, usize>,
    classes: HashMap<DefaultSymbol, Vec<usize>>,
    interner: RwLock<StringInterner>,
    selector_cache: RwLock<HashMap<String, Selector>>,
    source_map: SourceMap,
    source: String,
}

impl DOMIndex {
    pub fn new(dom: &markup5ever_rcdom::RcDom, source: &str) -> Self {
        let mut index = Self {
            arena: NodeArena::new(),
            elements: HashMap::with_capacity(256),
            ids: HashMap::with_capacity(256),
            classes: HashMap::with_capacity(256),
            interner: RwLock::new(StringInterner::with_capacity(1024)),
            selector_cache: RwLock::new(HashMap::with_capacity(64)),
            source_map: SourceMap::new(source),
            source: source.to_string(),
        };

        index.build_from_node(&dom.document);
        index
    }

    pub fn query(&self, selector: &str) -> Vec<usize> {
        // Fast path: check cache first with read lock
        let selector = {
            let cache = self.selector_cache.read();
            match cache.get(selector) {
                Some(sel) => sel.clone(),
                None => {
                    drop(cache);
                    let sel = self.parse_selector(selector);
                    self.selector_cache
                        .write()
                        .insert(selector.to_string(), sel.clone());
                    sel
                }
            }
        };

        // Collect matches from all alternatives
        let mut results = Vec::new();
        for alt in &selector.alternatives {
            // Optimize query path selection based on selector specificity
            let initial_set = if alt.element.is_none()
                && alt.id.is_none()
                && alt.classes.is_empty()
                && alt.attributes.is_empty()
            {
                // Handle universal "*" selector - match all elements
                (0..self.arena.nodes.len()).collect()
            } else if let Some(id) = alt.id {
                self.ids.get(&id).map(|&idx| vec![idx]).unwrap_or_default()
            } else if let Some(element) = alt.element {
                self.elements.get(&element).cloned().unwrap_or_default()
            } else if !alt.classes.is_empty() {
                // Start with smallest class set for better performance
                alt.classes
                    .iter()
                    .filter_map(|class| self.classes.get(class))
                    .min_by_key(|v| v.len())
                    .cloned()
                    .unwrap_or_default()
            } else {
                (0..self.arena.nodes.len()).collect()
            };

            // Apply remaining filters
            let matches: Vec<usize> = initial_set
                .into_iter()
                .filter(|&idx| {
                    let node = unsafe { self.arena.nodes.get_unchecked(idx) };

                    // Check classes
                    let classes_match =
                        alt.classes.iter().all(|class| node.classes.contains(class));

                    // Check attributes
                    let attrs_match = alt.attributes.iter().all(|attr_sel| match attr_sel {
                        AttributeSelector::Exists(attr_name) => {
                            node.attributes.iter().any(|a| a.name == *attr_name)
                        }
                        AttributeSelector::Equals(attr_name, value) => node
                            .attributes
                            .iter()
                            .any(|a| a.name == *attr_name && a.value == *value),
                        AttributeSelector::StartsWith(attr_name, value) => {
                            node.attributes.iter().any(|a| {
                                if a.name == *attr_name {
                                    let interner = self.interner.read();
                                    let attr_str = interner.resolve(a.value).unwrap();
                                    let value_str = interner.resolve(*value).unwrap();
                                    attr_str.starts_with(value_str)
                                } else {
                                    false
                                }
                            })
                        }
                        AttributeSelector::EndsWith(attr_name, value) => {
                            node.attributes.iter().any(|a| {
                                if a.name == *attr_name {
                                    let interner = self.interner.read();
                                    let attr_str = interner.resolve(a.value).unwrap();
                                    let value_str = interner.resolve(*value).unwrap();
                                    attr_str.ends_with(value_str)
                                } else {
                                    false
                                }
                            })
                        }
                        AttributeSelector::Contains(attr_name, value) => {
                            node.attributes.iter().any(|a| {
                                if a.name == *attr_name {
                                    let interner = self.interner.read();
                                    let attr_str = interner.resolve(a.value).unwrap();
                                    let value_str = interner.resolve(*value).unwrap();
                                    attr_str.contains(value_str)
                                } else {
                                    false
                                }
                            })
                        }
                    });

                    classes_match && attrs_match
                })
                .collect();

            results.extend(matches);
        }

        // Remove duplicates that might occur from multiple matching alternatives
        results.sort_unstable();
        results.dedup();
        results
    }

    fn parse_selector(&self, selector: &str) -> Selector {
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
                            element = Some(self.interner.write().get_or_intern(&token));
                            token.clear();
                        }

                        // Parse attribute name
                        while let Some(&c) = chars.peek() {
                            if c == '=' || c == '^' || c == '$' || c == '*' || c == ']' {
                                break;
                            }
                            token.push(chars.next().unwrap());
                        }
                        let attr_name = self.interner.write().get_or_intern(&token);
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

                                let value = self.interner.write().get_or_intern(&token);
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

                                        let value = self.interner.write().get_or_intern(&token);
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
                            element = Some(self.interner.write().get_or_intern(&token));
                            token.clear();
                        }
                        while let Some(&c) = chars.peek() {
                            if c == '.' || c == '#' {
                                break;
                            }
                            token.push(chars.next().unwrap());
                        }
                        id = Some(self.interner.write().get_or_intern(&token));
                        token.clear();
                    }
                    '.' => {
                        if !token.is_empty() {
                            element = Some(self.interner.write().get_or_intern(&token));
                            token.clear();
                        }
                        while let Some(&c) = chars.peek() {
                            if c == '.' || c == '#' {
                                break;
                            }
                            token.push(chars.next().unwrap());
                        }
                        classes.push(self.interner.write().get_or_intern(&token));
                        token.clear();
                    }
                    _ => token.push(c),
                }
            }

            if !token.is_empty() {
                element = Some(self.interner.write().get_or_intern(&token));
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

    fn build_from_node(&mut self, handle: &markup5ever_rcdom::Handle) -> usize {
        let idx = self.arena.nodes.len();
        let node = self.arena.allocate();
        node.handle = Some(handle.clone());

        match &handle.data {
            markup5ever_rcdom::NodeData::Element { name, attrs, .. } => {
                // Extract source info from the node
                let source_text = Self::extract_node_source(handle);
                let tag = self.interner.write().get_or_intern(&name.local);
                node.tag_name = tag;
                self.elements.entry(tag).or_default().push(idx);

                if let Some(source_text) = source_text {
                    if let Some(offset) = self.source.find(&source_text) {
                        let (line, column) = self.source_map.get_position(offset);
                        node.source_info = SourceInfo {
                            line,
                            column,
                            source: source_text,
                        };
                    }
                }

                for attr in attrs.borrow().iter() {
                    let name = self.interner.write().get_or_intern(&attr.name.local);
                    let value = self.interner.write().get_or_intern(&attr.value);

                    match &*attr.name.local {
                        "id" => {
                            self.ids.insert(value, idx);
                        }
                        "class" => {
                            for class in attr.value.split_whitespace() {
                                let class_sym = self.interner.write().get_or_intern(class);
                                node.classes.push(class_sym);
                                self.classes.entry(class_sym).or_default().push(idx);
                            }
                        }
                        _ => {}
                    }

                    node.attributes.push(IndexedAttribute {
                        name,
                        value,
                        quotes_type: if attr.value.contains('\'') {
                            QuotesType::Single
                        } else {
                            QuotesType::Double
                        },
                    });
                }
            }
            markup5ever_rcdom::NodeData::Text { contents } => {
                let text = contents.borrow();
                if !text.trim().is_empty() {
                    node.text_content =
                        Some(self.interner.write().get_or_intern(&text.to_string()));
                }
            }
            _ => {}
        }

        for child in handle.children.borrow().iter() {
            let child_idx = self.build_from_node(child);
            if let Some(child_node) = self.arena.get_mut(child_idx) {
                child_node.parent = Some(idx);
            }
        }

        idx
    }

    fn extract_node_source(handle: &markup5ever_rcdom::Handle) -> Option<String> {
        match &handle.data {
            markup5ever_rcdom::NodeData::Element { name, attrs, .. } => {
                let mut source = String::new();
                source.push('<');
                source.push_str(&name.local);

                for attr in attrs.borrow().iter() {
                    source.push(' ');
                    source.push_str(&attr.name.local);
                    source.push('=');
                    match attr.value.contains('\'') {
                        true => {
                            source.push('"');
                            source.push_str(&attr.value);
                            source.push('"');
                        }
                        false => {
                            source.push('\'');
                            source.push_str(&attr.value);
                            source.push('\'');
                        }
                    }
                }
                source.push('>');
                Some(source)
            }
            markup5ever_rcdom::NodeData::Text { contents } => Some(contents.borrow().to_string()),
            _ => None,
        }
    }

    pub fn get_node(&self, index: usize) -> Option<&IndexedNode> {
        self.arena.get(index)
    }

    pub fn get_nodes(&self) -> &[IndexedNode] {
        &self.arena.nodes
    }

    pub fn resolve_symbol(&self, symbol: DefaultSymbol) -> Option<String> {
        self.interner.read().resolve(symbol).map(|s| s.to_string())
    }

    pub fn get_source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn has_doctype(&self) -> bool {
        // Check if any direct child of the document is a DOCTYPE declaration
        if let Some(document) = self.get_node(0) {
            if let Some(handle) = &document.handle {
                for child in handle.children.borrow().iter() {
                    if let markup5ever_rcdom::NodeData::Doctype { .. } = child.data {
                        return true;
                    }
                }
            }
        }
        false
    }
}
