use super::*;
use markup5ever_rcdom::{Handle, NodeData};

pub(crate) trait NodeExt {
    fn get_tag_name(&self) -> Option<&str>;
    fn get_attributes(&self) -> Vec<(String, String)>;
    fn get_text_content(&self) -> String;
}

impl NodeExt for Handle {
    fn get_tag_name(&self) -> Option<&str> {
        if let NodeData::Element { ref name, .. } = self.data {
            Some(&name.local)
        } else {
            None
        }
    }

    fn get_attributes(&self) -> Vec<(String, String)> {
        if let NodeData::Element { ref attrs, .. } = self.data {
            let attrs = attrs.borrow();
            attrs
                .iter()
                .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_text_content(&self) -> String {
        let mut content = String::new();
        extract_text(self, &mut content);
        content
    }
}

pub(crate) fn extract_text(handle: &Handle, output: &mut String) {
    match handle.data {
        NodeData::Text { ref contents } => {
            output.push_str(&contents.borrow());
        }
        _ => {
            for child in handle.children.borrow().iter() {
                extract_text(child, output);
            }
        }
    }
}

pub(crate) fn find_node_position(source: &str, node_source: &str) -> Option<(usize, usize)> {
    if let Some(offset) = source.find(node_source) {
        let prefix = &source[..offset];
        let line = prefix.matches('\n').count() + 1;
        let column = offset - prefix.rfind('\n').map_or(0, |i| i + 1) + 1;
        Some((line, column))
    } else {
        None
    }
}

pub(crate) fn has_parent_of_type(
    node: &IndexedNode,
    parent_type: string_interner::DefaultSymbol,
    index: &DOMIndex,
) -> bool {
    let mut current = Some(node.parent);
    while let Some(Some(parent_idx)) = current {
        if let Some(parent) = index.get_node(parent_idx) {
            if parent.tag_name == parent_type {
                return true;
            }
            current = Some(parent.parent);
        } else {
            break;
        }
    }
    false
}

pub(crate) fn get_node_ancestors<'a>(
    node: &'a IndexedNode,
    index: &'a DOMIndex,
) -> Vec<&'a IndexedNode> {
    let mut ancestors = Vec::new();
    let mut current = Some(node.parent);

    while let Some(Some(parent_idx)) = current {
        if let Some(parent) = index.get_node(parent_idx) {
            ancestors.push(parent);
            current = Some(parent.parent);
        } else {
            break;
        }
    }

    ancestors
}

pub(crate) fn get_node_siblings<'a>(
    node: &'a IndexedNode,
    index: &'a DOMIndex,
) -> Vec<&'a IndexedNode> {
    if let Some(parent_idx) = node.parent {
        if let Some(parent) = index.get_node(parent_idx) {
            return parent
                .children
                .iter()
                .filter(|&&child_idx| child_idx != node.parent.unwrap())
                .filter_map(|&child_idx| index.get_node(child_idx))
                .collect();
        }
    }
    Vec::new()
}

pub(crate) fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub(crate) fn is_block_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "canvas"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "noscript"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "tfoot"
            | "ul"
            | "video"
    )
}

pub(crate) fn get_attribute<'a>(
    node: &'a IndexedNode,
    name: string_interner::DefaultSymbol,
    index: &'a DOMIndex,
) -> Option<&'a IndexedAttribute> {
    node.attributes.iter().find(|attr| attr.name == name)
}

pub(crate) fn has_class(node: &IndexedNode, class: string_interner::DefaultSymbol) -> bool {
    node.classes.contains(&class)
}

pub(crate) fn get_node_text_content(node_idx: usize, index: &DOMIndex) -> String {
    let mut content = String::new();
    collect_node_text(node_idx, index, &mut content);
    content.trim().to_string()
}

fn collect_node_text(node_idx: usize, index: &DOMIndex, output: &mut String) {
    let node = &index.get_node(node_idx).unwrap();

    // First check if this node has text content
    if let Some(text_symbol) = node.text_content {
        if let Some(text) = index.resolve_symbol(text_symbol) {
            output.push_str(&text);
            output.push(' '); // Add space between text nodes
        }
    }

    // Then recurse through children
    for &child_idx in &node.children {
        collect_node_text(child_idx, index, output);
    }
}

pub(crate) fn get_heading_level(node: &IndexedNode, index: &DOMIndex) -> Option<i32> {
    let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();
    if !tag_name.starts_with('h') {
        return None;
    }

    tag_name[1..]
        .parse::<i32>()
        .ok()
        .filter(|&level| level >= 1 && level <= 6)
}

pub(crate) fn is_sectioning_element(tag_name: &str) -> bool {
    matches!(tag_name, "article" | "section" | "aside" | "nav")
}

pub(crate) fn get_node_depth(node_idx: usize, index: &DOMIndex) -> usize {
    let mut depth = 0;
    let mut current = Some(node_idx);

    while let Some(idx) = current {
        depth += 1;
        current = index.get_node(idx).and_then(|node| node.parent);
    }

    depth
}
