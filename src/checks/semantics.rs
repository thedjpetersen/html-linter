use crate::*;

impl HtmlLinter {
    pub(crate) fn check_semantics(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(_node) = index.get_node(node_idx) {
                let should_report = match rule.condition.as_str() {
                    "semantic-elements" => self.check_semantic_elements(rule, index)?,
                    "semantic-landmarks" => self.check_semantic_landmarks(node_idx, index),
                    "semantic-buttons" => self.check_semantic_buttons(node_idx, index),
                    "semantic-tables" => self.check_semantic_tables(node_idx, index),
                    _ => vec![],
                };

                results.extend(should_report);
            }
        }

        Ok(results)
    }

    fn check_semantic_landmarks(&self, node_idx: usize, index: &DOMIndex) -> Vec<LintResult> {
        let mut results = Vec::new();

        if let Some(node) = index.get_node(node_idx) {
            let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();

            // Check if div/span is used where a semantic landmark element would be more appropriate
            if tag_name == "div" || tag_name == "span" {
                let has_landmark_class = node.attributes.iter().any(|attr| {
                    let class = index.resolve_symbol(attr.value).unwrap_or_default();
                    ["header", "footer", "nav", "main", "aside", "article"]
                        .iter()
                        .any(|&landmark| class.contains(landmark))
                });

                if has_landmark_class {
                    results.push(LintResult {
                        rule: "semantic-landmarks".to_string(),
                        severity: Severity::Warning,
                        message: "Consider using semantic landmark elements instead of div/span with landmark classes".to_string(),
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: tag_name.to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        results
    }

    fn check_semantic_buttons(&self, node_idx: usize, index: &DOMIndex) -> Vec<LintResult> {
        let mut results = Vec::new();

        if let Some(node) = index.get_node(node_idx) {
            let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();

            // Check if div/span is used as a button
            if tag_name == "div" || tag_name == "span" {
                let has_button_attributes = node.attributes.iter().any(|attr| {
                    let name = index.resolve_symbol(attr.name).unwrap_or_default();
                    let value = index.resolve_symbol(attr.value).unwrap_or_default();
                    (name == "onclick" || name == "role" && value == "button")
                });

                if has_button_attributes {
                    results.push(LintResult {
                        rule: "semantic-buttons".to_string(),
                        severity: Severity::Warning,
                        message: "Use <button> element instead of div/span with button behavior"
                            .to_string(),
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: tag_name.to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        results
    }

    fn check_semantic_tables(&self, node_idx: usize, index: &DOMIndex) -> Vec<LintResult> {
        let mut results = Vec::new();

        if let Some(node) = index.get_node(node_idx) {
            let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();

            if tag_name == "table" {
                // Check for presence of th elements
                let has_headers = node.children.iter().any(|&child_idx| {
                    if let Some(child) = index.get_node(child_idx) {
                        index.resolve_symbol(child.tag_name).unwrap_or_default() == "th"
                    } else {
                        false
                    }
                });

                // Check for caption
                let has_caption = node.children.iter().any(|&child_idx| {
                    if let Some(child) = index.get_node(child_idx) {
                        index.resolve_symbol(child.tag_name).unwrap_or_default() == "caption"
                    } else {
                        false
                    }
                });

                if !has_headers || !has_caption {
                    results.push(LintResult {
                        rule: "semantic-tables".to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "Table is missing semantic elements: {}",
                            if !has_headers && !has_caption {
                                "headers (th) and caption"
                            } else if !has_headers {
                                "headers (th)"
                            } else {
                                "caption"
                            }
                        ),
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: tag_name.to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        results
    }

    fn check_semantic_elements(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();

        // Check for common non-semantic pattern replacements
        let patterns = [
            ("b", "strong", "Use <strong> for strong importance"),
            ("i", "em", "Use <em> for emphasized text"),
            (
                "div[role=button]",
                "button",
                "Use <button> instead of div with button role",
            ),
            (
                "div[role=navigation]",
                "nav",
                "Use <nav> instead of div with navigation role",
            ),
            (
                "div[role=main]",
                "main",
                "Use <main> instead of div with main role",
            ),
        ];

        for (non_semantic, _semantic, message) in patterns {
            let matches = index.query(non_semantic);
            for node_idx in matches {
                if let Some(node) = index.get_node(node_idx) {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: message.to_string(),
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
}
