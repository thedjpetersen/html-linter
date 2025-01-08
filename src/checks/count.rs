use crate::*;

impl HtmlLinter {
    pub(crate) fn check_element_count(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        match rule.condition.as_str() {
            "max-count" => {
                let max_count: usize = rule
                    .options
                    .get("max")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);

                if matches.len() > max_count {
                    if let Some(&node_idx) = matches.get(max_count) {
                        if let Some(node) = index.get_node(node_idx) {
                            results.push(self.create_lint_result(rule, node, index));
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(results)
    }

    pub(crate) fn check_element_case(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let element_name = index.resolve_symbol(node.tag_name).unwrap_or_default();
                let has_uppercase = element_name.chars().any(|c| c.is_uppercase());

                let uppercase_attrs: Vec<_> = node
                    .attributes
                    .iter()
                    .filter_map(|attr| {
                        let name = index.resolve_symbol(attr.name).unwrap_or_default();
                        if name.chars().any(|c| c.is_uppercase()) {
                            Some(name.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                if has_uppercase || !uppercase_attrs.is_empty() {
                    let mut message = rule.message.clone();
                    if has_uppercase {
                        message.push_str(&format!(" (element: {})", element_name));
                    }
                    if !uppercase_attrs.is_empty() {
                        message.push_str(&format!(" (attributes: {})", uppercase_attrs.join(", ")));
                    }

                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message,
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: element_name.to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        Ok(results)
    }
}
