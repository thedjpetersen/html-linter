use crate::*;

impl HtmlLinter {
    pub(crate) fn check_element_order(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();

        if rule.condition == "sequential-order" {
            let mut heading_stack = Vec::new();

            // More efficient iteration using direct node access
            for node_idx in 0..index.get_nodes().len() {
                if let Some(node) = index.get_node(node_idx) {
                    let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();

                    // Check if it's a heading and parse level
                    if let Some(level) = parse_heading_level(&tag_name) {
                        match heading_stack.last() {
                            Some(&prev_level) => {
                                // Check for skipped heading levels
                                if level > prev_level + 1 {
                                    results.push(LintResult {
                                        rule: rule.name.clone(),
                                        severity: rule.severity.clone(),
                                        message: format!(
                                            "Heading level jumped from h{} to h{}",
                                            prev_level, level
                                        ),
                                        location: Location {
                                            line: node.source_info.line,
                                            column: node.source_info.column,
                                            element: tag_name.clone(),
                                        },
                                        source: node.source_info.source.clone(),
                                    });
                                }

                                // Handle heading level changes
                                if level > prev_level {
                                    heading_stack.push(level);
                                } else {
                                    while let Some(&stack_level) = heading_stack.last() {
                                        if stack_level >= level {
                                            heading_stack.pop();
                                        } else {
                                            break;
                                        }
                                    }
                                    heading_stack.push(level);
                                }
                            }
                            None => heading_stack.push(level),
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub(crate) fn check_nesting(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let should_report = match rule.condition.as_str() {
                    "parent-label-or-for" => {
                        !self.has_label_parent(node_idx, index)
                            && !self.has_matching_label(node_idx, index)
                    }
                    _ => false,
                };

                if should_report {
                    results.push(self.create_lint_result(rule, node, index));
                }
            }
        }

        Ok(results)
    }

    pub(crate) fn check_document_structure(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();

        match rule.condition.as_str() {
            "doctype-present" => {
                let has_doctype = index
                    .get_source_map()
                    .lines
                    .iter()
                    .any(|line| line.trim().to_lowercase().starts_with("<!doctype"));

                if !has_doctype {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: rule.message.clone(),
                        location: Location {
                            line: 1,
                            column: 1,
                            element: String::new(),
                        },
                        source: String::new(),
                    });
                }
            }
            _ => {}
        }

        Ok(results)
    }

    fn has_label_parent(&self, node_idx: usize, index: &DOMIndex) -> bool {
        let mut current_idx = node_idx;
        while let Some(parent_idx) = index.get_node(current_idx).and_then(|n| n.parent) {
            if let Some(parent_node) = index.get_node(parent_idx) {
                if index
                    .resolve_symbol(parent_node.tag_name)
                    .unwrap_or_default()
                    == "label"
                {
                    return true;
                }
                current_idx = parent_idx;
            }
        }
        false
    }

    fn has_matching_label(&self, node_idx: usize, index: &DOMIndex) -> bool {
        // Get the ID of the current node
        if let Some(node) = index.get_node(node_idx) {
            let node_id = node.attributes.iter().find_map(|attr| {
                if index.resolve_symbol(attr.name).unwrap_or_default() == "id" {
                    Some(index.resolve_symbol(attr.value).unwrap_or_default())
                } else {
                    None
                }
            });

            // If the node has no ID, it can't have a matching label
            if let Some(id) = node_id {
                // Look for any label with a matching "for" attribute
                index.get_nodes().iter().any(|other_node| {
                    if index
                        .resolve_symbol(other_node.tag_name)
                        .unwrap_or_default()
                        == "label"
                    {
                        other_node.attributes.iter().any(|attr| {
                            index.resolve_symbol(attr.name).unwrap_or_default() == "for"
                                && index.resolve_symbol(attr.value).unwrap_or_default() == id
                        })
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        } else {
            false
        }
    }
}

// Helper function to safely parse heading levels
fn parse_heading_level(tag_name: &str) -> Option<i32> {
    if !tag_name.starts_with('h') {
        return None;
    }

    tag_name[1..]
        .parse::<i32>()
        .ok()
        .filter(|&level| level >= 1 && level <= 6)
}
