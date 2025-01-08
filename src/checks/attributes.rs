use dom::QuotesType;

use crate::*;

impl HtmlLinter {
    pub(crate) fn check_attribute_value(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        // Special handling for unique-id condition
        if rule.condition == "unique-id" {
            return self.check_unique_ids(rule, index);
        }

        // Special handling for positive-number condition
        if rule.condition == "positive-number" {
            return self.check_positive_number(rule, index);
        }

        let pattern = rule.options.get("pattern").ok_or_else(|| {
            LinterError::RuleError("Pattern option required for attribute value check".to_string())
        })?;

        let regex = Regex::new(pattern).map_err(|e| LinterError::RuleError(e.to_string()))?;

        let check_mode = rule
            .options
            .get("check_mode")
            .map(String::as_str)
            .unwrap_or("normal");

        let attributes: Vec<_> = rule
            .options
            .get("attributes")
            .map(|attrs| attrs.split(',').map(str::trim).collect())
            .unwrap_or_else(|| vec!["*"]);

        let matches = index.query(&rule.selector);
        let mut results = Vec::new();

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let (has_required_attr, found_match) =
                    self.check_node_attributes(node, index, &attributes, &regex);

                let should_report = match check_mode {
                    "ensure_existence" => !has_required_attr || !found_match,
                    "ensure_nonexistence" => has_required_attr && found_match,
                    _ => found_match,
                };

                if should_report {
                    results.push(self.create_lint_result(rule, node, index));
                }
            }
        }

        Ok(results)
    }

    fn check_node_attributes(
        &self,
        node: &IndexedNode,
        index: &DOMIndex,
        attributes: &[&str],
        regex: &Regex,
    ) -> (bool, bool) {
        let mut has_required_attr = false;
        let mut found_match = false;

        for attr in &node.attributes {
            let attr_name = index.resolve_symbol(attr.name).unwrap_or_default();
            if attributes.contains(&"*") || attributes.contains(&attr_name.as_str()) {
                has_required_attr = true;
                let attr_value = index.resolve_symbol(attr.value).unwrap_or_default();
                if regex.is_match(&attr_value) {
                    found_match = true;
                    break;
                }
            }
        }

        (has_required_attr, found_match)
    }

    pub(crate) fn check_attribute_quotes(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);
        let quote_style = rule
            .options
            .get("style")
            .map(String::as_str)
            .unwrap_or("double");

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                for attr in &node.attributes {
                    let wrong_quotes = match quote_style {
                        "double" => attr.quotes_type == QuotesType::Single,
                        "single" => attr.quotes_type == QuotesType::Double,
                        _ => false,
                    };

                    if wrong_quotes {
                        results.push(LintResult {
                            rule: rule.name.clone(),
                            severity: rule.severity.clone(),
                            message: format!("{} (expected {} quotes)", rule.message, quote_style),
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
        }

        Ok(results)
    }

    fn check_unique_ids(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                for attr in &node.attributes {
                    if index.resolve_symbol(attr.name).unwrap_or_default() == "id" {
                        let id = index.resolve_symbol(attr.value).unwrap_or_default();
                        if !seen_ids.insert(id.to_string()) {
                            results.push(self.create_lint_result(rule, node, index));
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    fn check_positive_number(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                for attr in &node.attributes {
                    let attr_name = index.resolve_symbol(attr.name).unwrap_or_default();
                    if attr_name == "tabindex" {
                        let value = index.resolve_symbol(attr.value).unwrap_or_default();
                        if let Ok(num) = value.parse::<i32>() {
                            if num > 0 {
                                results.push(self.create_lint_result(rule, node, index));
                            }
                        }
                    }
                }
            }
        }
        Ok(results)
    }
}
