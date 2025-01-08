use crate::*;

impl HtmlLinter {
    pub(crate) fn check_element_presence(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let should_report = match rule.condition.as_str() {
                    "required" => false,
                    "forbidden" => true,
                    "semantic-alternative-available" => {
                        !self.check_semantic_alternative(node_idx, index)
                    }
                    "element-present" => false,
                    "doctype-present" => !index.has_doctype(),
                    _ => false,
                };

                if should_report {
                    results.push(self.create_lint_result(rule, node, index));
                }
            }
        }

        Ok(results)
    }

    pub(crate) fn check_attribute_presence(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let should_report = match rule.condition.as_str() {
                    "duplicate-attributes" => Self::has_duplicate_attributes(node, index),
                    "alt-missing" => Self::is_attribute_missing(node, index, &rule.condition),
                    "style-attribute" if !self.options.allow_inline_styles => {
                        Self::has_style_attribute(node, index)
                    }
                    "alt-attribute" => Self::is_attribute_missing(node, index, "alt"),
                    "lang-attribute" => Self::is_attribute_missing(node, index, "lang"),
                    _ => false,
                };

                if should_report {
                    let message = if rule.condition == "duplicate-attributes" {
                        let mut duplicates = Vec::new();
                        let mut seen = std::collections::HashMap::new();

                        for attr in &node.attributes {
                            let name = index.resolve_symbol(attr.name).unwrap_or_default();
                            *seen.entry(name).or_insert(0) += 1;
                        }

                        for (name, count) in seen {
                            if count > 1 {
                                duplicates.push(format!("{} ({}×)", name, count));
                            }
                        }

                        format!("{} (duplicates: {})", rule.message, duplicates.join(", "))
                    } else {
                        rule.message.clone()
                    };

                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message,
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: index
                                .resolve_symbol(node.tag_name)
                                .unwrap_or_default()
                                .to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    fn check_semantic_alternative(&self, node_idx: usize, index: &DOMIndex) -> bool {
        if let Some(node) = index.get_node(node_idx) {
            let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();

            if tag_name == "div" || tag_name == "span" {
                return node.attributes.iter().any(|attr| {
                    let attr_name = index.resolve_symbol(attr.name).unwrap_or_default();
                    let attr_value = index.resolve_symbol(attr.value).unwrap_or_default();

                    attr_name == "role"
                        || (attr_name == "class"
                            && ["button", "navigation", "content", "header"]
                                .iter()
                                .any(|&semantic| attr_value.contains(semantic)))
                });
            }
        }
        false
    }

    #[inline]
    fn has_duplicate_attributes(node: &IndexedNode, index: &DOMIndex) -> bool {
        let source = &node.source_info.source;
        let mut seen_attributes = std::collections::HashMap::new();
        let mut pos = 0;
        let bytes = source.as_bytes();

        // Skip until we find the tag name
        while pos < bytes.len() && bytes[pos] != b'<' {
            pos += 1;
        }
        if pos >= bytes.len() {
            return false;
        }
        pos += 1;

        // Skip tag name
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        // Parse attributes
        while pos < bytes.len() {
            // Skip whitespace
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos >= bytes.len() || bytes[pos] == b'>' || bytes[pos] == b'/' {
                break;
            }

            // Read attribute name
            let start = pos;
            while pos < bytes.len()
                && !bytes[pos].is_ascii_whitespace()
                && bytes[pos] != b'='
                && bytes[pos] != b'>'
            {
                pos += 1;
            }
            if start == pos {
                break;
            }

            // Safe to use from_utf8_unchecked as we only included ASCII chars
            let attr_name = unsafe { std::str::from_utf8_unchecked(&bytes[start..pos]) };

            // Count attribute occurrence
            *seen_attributes.entry(attr_name.to_string()).or_insert(0) += 1;
            if seen_attributes[attr_name] > 1 {
                return true;
            }

            // Skip attribute value if present
            if pos < bytes.len() && bytes[pos] == b'=' {
                pos += 1;
                // Skip whitespace after =
                while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                if pos < bytes.len() {
                    match bytes[pos] {
                        b'"' | b'\'' => {
                            let quote = bytes[pos];
                            pos += 1;
                            while pos < bytes.len() && bytes[pos] != quote {
                                pos += 1;
                            }
                            if pos < bytes.len() {
                                pos += 1;
                            }
                        }
                        _ => {
                            while pos < bytes.len()
                                && !bytes[pos].is_ascii_whitespace()
                                && bytes[pos] != b'>'
                            {
                                pos += 1;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    #[inline]
    fn is_attribute_missing(node: &IndexedNode, index: &DOMIndex, condition: &str) -> bool {
        let attr_name = condition.split('-').next().unwrap_or("");
        !node
            .attributes
            .iter()
            .any(|attr| index.resolve_symbol(attr.name).unwrap_or_default() == attr_name)
    }

    #[inline]
    fn has_style_attribute(node: &IndexedNode, index: &DOMIndex) -> bool {
        node.attributes
            .iter()
            .any(|attr| index.resolve_symbol(attr.name).unwrap_or_default() == "style")
    }
}
