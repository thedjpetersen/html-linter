use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use string_interner::{DefaultSymbol, StringInterner};

// Move selector-related structs
#[derive(Clone, Debug, PartialEq)]
pub enum Combinator {
    Descendant,     // space
    Child,          // >
    Adjacent,       // +
    GeneralSibling, // ~
}

#[derive(Clone, Debug, PartialEq)]
pub enum PseudoClass {
    FirstChild,
    LastChild,
    NthChild(i32, i32), // an + b pattern
    NthLastChild(i32, i32),
    FirstOfType,
    LastOfType,
    OnlyChild,
    OnlyOfType,
    Empty,
    Not(Box<SelectorPart>),
}

// Expand attribute selectors
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeSelector {
    Exists(DefaultSymbol),                      // [attr]
    Equals(DefaultSymbol, DefaultSymbol),       // [attr=value]
    StartsWith(DefaultSymbol, DefaultSymbol),   // [attr^=value]
    EndsWith(DefaultSymbol, DefaultSymbol),     // [attr$=value]
    Contains(DefaultSymbol, DefaultSymbol),     // [attr*=value]
    ListContains(DefaultSymbol, DefaultSymbol), // [attr~=value]
    DashMatch(DefaultSymbol, DefaultSymbol),    // [attr|=value]
}

// Modify SelectorPart to include new features
#[derive(Clone, Debug, PartialEq)]
pub struct SelectorPart {
    pub(crate) element: Option<DefaultSymbol>,
    pub(crate) classes: Vec<DefaultSymbol>,
    pub(crate) id: Option<DefaultSymbol>,
    pub(crate) attributes: Vec<AttributeSelector>,
    pub(crate) pseudo_classes: Vec<PseudoClass>,
    pub(crate) combinator: Option<Combinator>,
    pub(crate) specificity: (u32, u32, u32), // (id_count, class_count, element_count)
}

// Add specificity calculation
impl SelectorPart {
    fn calculate_specificity(&mut self) {
        let id_count = self.id.is_some() as u32;
        let class_count = self.classes.len() as u32 + self.attributes.len() as u32;
        let element_count = self.element.is_some() as u32;
        self.specificity = (id_count, class_count, element_count);
    }
}

// Modify Selector struct to handle sequences
#[derive(Clone, Debug)]
pub struct Selector {
    pub(crate) alternatives: Vec<Vec<SelectorPart>>, // Each inner Vec represents a sequence
}

pub struct SelectorEngine {
    selector_cache: RwLock<HashMap<String, Selector>>,
    interner: RwLock<StringInterner>,
}

impl SelectorEngine {
    pub fn new(interner: StringInterner) -> Self {
        Self {
            selector_cache: RwLock::new(HashMap::with_capacity(64)),
            interner: RwLock::new(interner),
        }
    }

    fn parse_combinator(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Option<Combinator> {
        match chars.peek() {
            Some('>') => {
                chars.next();
                Some(Combinator::Child)
            }
            Some('+') => {
                chars.next();
                Some(Combinator::Adjacent)
            }
            Some('~') => {
                chars.next();
                Some(Combinator::GeneralSibling)
            }
            Some(' ') => {
                chars.next();
                Some(Combinator::Descendant)
            }
            _ => None,
        }
    }

    fn parse_pseudo_class(
        &self,
        name: &str,
        _chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Option<PseudoClass> {
        match name {
            "first-child" => Some(PseudoClass::FirstChild),
            "last-child" => Some(PseudoClass::LastChild),
            "nth-child" => {
                // Parse an+b pattern
                // Implementation needed
                Some(PseudoClass::NthChild(1, 0))
            }
            // Add other pseudo-class parsing...
            _ => None,
        }
    }

    fn parse_attribute_selector(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        interner: &RwLock<StringInterner>,
    ) -> Option<AttributeSelector> {
        let mut token = String::with_capacity(32);

        // Parse attribute name
        while let Some(&c) = chars.peek() {
            if c == '=' || c == '^' || c == '$' || c == '*' || c == '~' || c == '|' || c == ']' {
                break;
            }
            token.push(chars.next().unwrap());
        }
        let attr_name = interner.write().get_or_intern(&token.trim());
        token.clear();

        // Parse operator and value if present
        match chars.next() {
            Some(']') => Some(AttributeSelector::Exists(attr_name)),
            Some('=') => {
                let value = self.parse_attribute_value(chars);
                Some(AttributeSelector::Equals(
                    attr_name,
                    interner.write().get_or_intern(&value),
                ))
            }
            Some(c) => match c {
                '^' | '$' | '*' | '~' | '|' => {
                    if chars.next() != Some('=') {
                        return None;
                    }

                    let value = self.parse_attribute_value(chars);
                    let value_symbol = interner.write().get_or_intern(&value);

                    match c {
                        '^' => Some(AttributeSelector::StartsWith(attr_name, value_symbol)),
                        '$' => Some(AttributeSelector::EndsWith(attr_name, value_symbol)),
                        '*' => Some(AttributeSelector::Contains(attr_name, value_symbol)),
                        '~' => Some(AttributeSelector::ListContains(attr_name, value_symbol)),
                        '|' => Some(AttributeSelector::DashMatch(attr_name, value_symbol)),
                        _ => None,
                    }
                }
                _ => None,
            },
            None => None,
        }
    }

    fn parse_attribute_value(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut value = String::new();
        let mut in_quotes = false;
        let quote_char = match chars.peek() {
            Some(&'"') | Some(&'\'') => {
                in_quotes = true;
                chars.next()
            }
            _ => None,
        };

        while let Some(&c) = chars.peek() {
            if !in_quotes && (c == ']' || c == ' ') {
                break;
            }
            if in_quotes && Some(c) == quote_char {
                chars.next(); // consume closing quote
                break;
            }
            value.push(chars.next().unwrap());
        }

        // Skip closing bracket if present
        while let Some(&c) = chars.peek() {
            if c == ']' {
                chars.next();
                break;
            }
            if !c.is_whitespace() {
                break;
            }
            chars.next();
        }

        value
    }

    pub fn parse_selector(&self, selector: &str, interner: &RwLock<StringInterner>) -> Selector {
        // Handle universal selector "*" explicitly
        if selector == "*" {
            return Selector {
                alternatives: vec![vec![SelectorPart {
                    element: None,
                    classes: Vec::new(),
                    id: None,
                    attributes: Vec::new(),
                    pseudo_classes: Vec::new(),
                    combinator: None,
                    specificity: (0, 0, 0),
                }]],
            };
        }

        let mut alternatives = Vec::new();
        let mut current_sequence = Vec::new();

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

                        if let Some(attr_selector) =
                            self.parse_attribute_selector(&mut chars, interner)
                        {
                            attributes.push(attr_selector);
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

            current_sequence.push(SelectorPart {
                element,
                classes,
                id,
                attributes,
                pseudo_classes: Vec::new(),
                combinator: None,
                specificity: (0, 0, 0),
            });
        }

        alternatives.push(current_sequence);

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

    fn matches_pseudo_class(&self, element: &Element, pseudo: &PseudoClass) -> bool {
        match pseudo {
            PseudoClass::FirstChild => element.previous_sibling().is_none(),
            PseudoClass::LastChild => element.next_sibling().is_none(),
            PseudoClass::NthChild(a, b) => {
                let mut count = 1;
                let mut current = element.previous_sibling();
                while current.is_some() {
                    count += 1;
                    current = current.and_then(|e| e.borrow().previous_sibling());
                }
                (count - b) % a == 0 && count >= *b
            }
            PseudoClass::NthLastChild(a, b) => {
                let mut count = 1;
                let mut current = element.next_sibling();
                while current.is_some() {
                    count += 1;
                    current = current.and_then(|e| e.borrow().next_sibling());
                }
                (count - b) % a == 0 && count >= *b
            }
            PseudoClass::FirstOfType => {
                let tag_name = element.tag_name;
                let mut current = element.previous_sibling();
                while let Some(sibling) = current {
                    if let NodeData::Element(sibling_elem) = &*sibling.borrow() {
                        if sibling_elem.tag_name == tag_name {
                            return false;
                        }
                    }
                    current = sibling.borrow().previous_sibling();
                }
                true
            }
            PseudoClass::LastOfType => {
                let tag_name = element.tag_name;
                let mut current = element.next_sibling();
                while let Some(sibling) = current {
                    if let NodeData::Element(sibling_elem) = &*sibling.borrow() {
                        if sibling_elem.tag_name == tag_name {
                            return false;
                        }
                    }
                    current = sibling.borrow().next_sibling();
                }
                true
            }
            PseudoClass::OnlyChild => {
                element.previous_sibling().is_none() && element.next_sibling().is_none()
            }
            PseudoClass::OnlyOfType => {
                let tag_name = element.tag_name;
                self.matches_selector_sequence(
                    element.clone(),
                    &[SelectorPart {
                        element: Some(tag_name),
                        classes: Vec::new(),
                        id: None,
                        attributes: Vec::new(),
                        pseudo_classes: vec![PseudoClass::FirstOfType],
                        combinator: None,
                        specificity: (0, 0, 0),
                    }],
                )
            }
            PseudoClass::Empty => element.children.is_empty() && element.text.is_empty(),
            PseudoClass::Not(selector_part) => !self.matches_part(element, &*selector_part),
        }
    }

    fn matches_combinator(
        &self,
        element: &Element,
        reference: &Element,
        combinator: &Combinator,
    ) -> bool {
        match combinator {
            Combinator::Descendant => {
                // Check if reference is an ancestor of element
                reference.is_ancestor_of(element)
            }
            Combinator::Child => {
                // Check if reference is the direct parent
                if let Some(parent) = element.parent() {
                    let parent_ref = parent.borrow();
                    if let NodeData::Element(parent_elem) = &*parent_ref {
                        std::ptr::eq(parent_elem as *const Element, reference as *const Element)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Combinator::Adjacent => {
                // Check if reference is the immediate previous sibling
                if let Some(prev) = element.previous_sibling() {
                    let prev_ref = prev.borrow();
                    if let NodeData::Element(prev_elem) = &*prev_ref {
                        std::ptr::eq(prev_elem as *const Element, reference as *const Element)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Combinator::GeneralSibling => {
                // Check all previous siblings until we find a match or run out
                let mut current = element.previous_sibling();
                while let Some(sibling) = current {
                    let sibling_ref = sibling.borrow();
                    if let NodeData::Element(sibling_elem) = &*sibling_ref {
                        if std::ptr::eq(sibling_elem as *const Element, reference as *const Element)
                        {
                            return true;
                        }
                    }
                    current = sibling_ref.previous_sibling();
                }
                false
            }
        }
    }

    // Add a method to match a complete selector sequence
    fn matches_selector_sequence(&self, element: Element, sequence: &[SelectorPart]) -> bool {
        // Start with the rightmost (most specific) part
        let last_part = match sequence.iter().rev().next() {
            Some(part) => part,
            None => return false,
        };

        // Check if the current element matches the last part
        if !self.matches_part(&element, last_part) {
            return false;
        }

        let mut current_element = Some(element);
        let mut sequence_iter = sequence.iter().rev().skip(1); // Skip the part we already checked

        // Now check the rest of the sequence
        for part in sequence_iter {
            match &part.combinator {
                Some(combinator) => {
                    // Move to the next element according to combinator
                    current_element = match current_element {
                        Some(elem) => self.find_matching_ancestor(&elem, part, combinator),
                        None => return false,
                    };

                    if current_element.is_none() {
                        return false;
                    }
                }
                None => {
                    // Implicit descendant combinator
                    current_element = match current_element {
                        Some(elem) => {
                            self.find_matching_ancestor(&elem, part, &Combinator::Descendant)
                        }
                        None => return false,
                    };

                    if current_element.is_none() {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn find_matching_ancestor(
        &self,
        element: &Element,
        part: &SelectorPart,
        combinator: &Combinator,
    ) -> Option<Element> {
        match combinator {
            Combinator::Descendant => {
                let mut current = element.parent();
                while let Some(parent) = current {
                    let parent_ref = parent.borrow();
                    if let NodeData::Element(parent_elem) = &*parent_ref {
                        if self.matches_part(parent_elem, part) {
                            return Some(parent_elem.clone());
                        }
                        current = parent_elem.parent();
                    } else {
                        current = None;
                    }
                }
                None
            }
            Combinator::Child => element.parent().and_then(|parent| {
                let parent_ref = parent.borrow();
                if let NodeData::Element(parent_elem) = &*parent_ref {
                    if self.matches_part(parent_elem, part) {
                        Some(parent_elem.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            Combinator::Adjacent => element.previous_sibling().and_then(|sibling| {
                let sibling_ref = sibling.borrow();
                if let NodeData::Element(sibling_elem) = &*sibling_ref {
                    if self.matches_part(sibling_elem, part) {
                        Some(sibling_elem.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            Combinator::GeneralSibling => {
                let mut current = element.previous_sibling();
                while let Some(sibling) = current {
                    let sibling_ref = sibling.borrow();
                    if let NodeData::Element(sibling_elem) = &*sibling_ref {
                        if self.matches_part(sibling_elem, part) {
                            return Some(sibling_elem.clone());
                        }
                        current = sibling_elem.previous_sibling();
                    } else {
                        current = None;
                    }
                }
                None
            }
        }
    }

    fn matches_part(&self, element: &Element, part: &SelectorPart) -> bool {
        // Check element type
        if let Some(tag_name) = &part.element {
            if element.tag_name != *tag_name {
                return false;
            }
        }

        // Check ID
        if let Some(id) = &part.id {
            if !element.has_id(*id) {
                return false;
            }
        }

        // Check classes
        for class in &part.classes {
            if !element.has_class(*class) {
                return false;
            }
        }

        // Check attributes
        for attr in &part.attributes {
            if !self.matches_attribute(element, attr) {
                return false;
            }
        }

        // Check pseudo-classes
        for pseudo in &part.pseudo_classes {
            if !self.matches_pseudo_class(element, pseudo) {
                return false;
            }
        }

        true
    }

    fn matches_attribute(&self, element: &Element, attr_selector: &AttributeSelector) -> bool {
        match attr_selector {
            AttributeSelector::Exists(name) => element.has_attribute(*name),
            AttributeSelector::Equals(name, value) => element.get_attribute(*name) == Some(*value),
            AttributeSelector::StartsWith(name, value) => {
                if let Some(attr_value) = element.get_attribute(*name) {
                    let interner = self.interner.read();
                    let attr_str = interner.resolve(attr_value).unwrap();
                    let value_str = interner.resolve(*value).unwrap();
                    attr_str.starts_with(value_str)
                } else {
                    false
                }
            }
            AttributeSelector::EndsWith(name, value) => {
                if let Some(attr_value) = element.get_attribute(*name) {
                    let interner = self.interner.read();
                    let attr_str = interner.resolve(attr_value).unwrap();
                    let value_str = interner.resolve(*value).unwrap();
                    attr_str.ends_with(value_str)
                } else {
                    false
                }
            }
            AttributeSelector::Contains(name, value) => {
                if let Some(attr_value) = element.get_attribute(*name) {
                    let interner = self.interner.read();
                    let attr_str = interner.resolve(attr_value).unwrap();
                    let value_str = interner.resolve(*value).unwrap();
                    attr_str.contains(value_str)
                } else {
                    false
                }
            }
            AttributeSelector::ListContains(name, value) => {
                element.get_attribute(*name).map_or(false, |attr| {
                    let interner = self.interner.read();
                    let attr_str = interner.resolve(attr).unwrap();
                    let value_str = interner.resolve(*value).unwrap();
                    attr_str.contains(value_str)
                })
            }
            AttributeSelector::DashMatch(name, value) => {
                element.get_attribute(*name).map_or(false, |attr| {
                    let interner = self.interner.read();
                    let attr_str = interner.resolve(attr).unwrap();
                    let value_str = interner.resolve(*value).unwrap();
                    attr_str == value_str || attr_str.starts_with(&format!("{:?}-", value_str))
                })
            }
        }
    }
}

// First, let's add a helper trait for node traversal
pub(crate) trait NodeTraversal {
    fn parent(&self) -> Option<Rc<RefCell<NodeData>>>;
    fn previous_sibling(&self) -> Option<Rc<RefCell<NodeData>>>;
    fn next_sibling(&self) -> Option<Rc<RefCell<NodeData>>>;
    fn is_ancestor_of(&self, other: &NodeData) -> bool;
}

impl NodeTraversal for NodeData {
    fn parent(&self) -> Option<Rc<RefCell<NodeData>>> {
        match self {
            NodeData::Element(element) => element.parent.upgrade(),
            _ => None,
        }
    }

    fn previous_sibling(&self) -> Option<Rc<RefCell<NodeData>>> {
        match self {
            NodeData::Element(element) => element.previous_sibling.upgrade(),
            _ => None,
        }
    }

    fn next_sibling(&self) -> Option<Rc<RefCell<NodeData>>> {
        match self {
            NodeData::Element(element) => element.next_sibling.as_ref().cloned(),
            _ => None,
        }
    }

    fn is_ancestor_of(&self, other: &NodeData) -> bool {
        let mut current = other.parent();
        while let Some(parent) = current {
            let parent_ref = parent.borrow();
            if let NodeData::Element(parent_elem) = &*parent_ref {
                if let NodeData::Element(self_elem) = self {
                    if std::ptr::eq(self_elem, parent_elem) {
                        return true;
                    }
                }
                current = parent_elem.parent();
            } else {
                current = None;
            }
        }
        false
    }
}

// Add Element and NodeData types
#[derive(Debug, Clone)]
pub struct Element {
    pub tag_name: DefaultSymbol,
    pub parent: std::rc::Weak<RefCell<NodeData>>,
    pub previous_sibling: std::rc::Weak<RefCell<NodeData>>,
    pub next_sibling: Option<Rc<RefCell<NodeData>>>,
    pub children: Vec<Rc<RefCell<NodeData>>>,
    pub attributes: HashSet<(DefaultSymbol, DefaultSymbol)>,
    pub classes: HashSet<DefaultSymbol>,
    pub id: Option<DefaultSymbol>,
    pub text: String,
}

#[derive(Debug)]
pub enum NodeData {
    Element(Element),
    Text(String),
    Comment(String),
    ProcessingInstruction(String, String),
    Doctype(String),
}

// Add Element implementation with required methods
impl Element {
    pub fn has_id(&self, id: DefaultSymbol) -> bool {
        self.id == Some(id)
    }

    pub fn has_class(&self, class: DefaultSymbol) -> bool {
        self.classes.contains(&class)
    }

    pub fn has_attribute(&self, name: DefaultSymbol) -> bool {
        self.attributes
            .iter()
            .any(|(attr_name, _)| *attr_name == name)
    }

    pub fn get_attribute(&self, name: DefaultSymbol) -> Option<DefaultSymbol> {
        self.attributes
            .iter()
            .find(|(attr_name, _)| *attr_name == name)
            .map(|(_, value)| *value)
    }

    pub fn parent(&self) -> Option<Rc<RefCell<NodeData>>> {
        self.parent.upgrade()
    }

    pub fn previous_sibling(&self) -> Option<Rc<RefCell<NodeData>>> {
        self.previous_sibling.upgrade()
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<NodeData>>> {
        self.next_sibling.clone()
    }

    pub fn is_ancestor_of(&self, other: &Element) -> bool {
        let mut current = other.parent();
        while let Some(parent) = current {
            let parent_ref = parent.borrow();
            if let NodeData::Element(parent_elem) = &*parent_ref {
                if std::ptr::eq(self, parent_elem) {
                    return true;
                }
                current = parent_elem.parent();
            } else {
                current = None;
            }
        }
        false
    }
}
