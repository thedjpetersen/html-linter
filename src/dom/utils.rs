use super::*;
use markup5ever_rcdom::{Handle, NodeData};

pub(crate) trait _NodeExt {
    fn get_tag_name(&self) -> Option<&str>;
    fn get_attributes(&self) -> Vec<(String, String)>;
    fn get_text_content(&self) -> String;
}

impl _NodeExt for Handle {
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
    if let NodeData::Text { ref contents } = handle.data {
        output.push_str(&contents.borrow());
    }

    // Only get direct text nodes, skip recursing into elements
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { .. } = child.data {
            extract_text(child, output);
        }
    }
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
