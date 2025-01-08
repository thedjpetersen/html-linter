use crate::*;
use markup5ever_rcdom::NodeData;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct MetaTagRule {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    property: Option<String>,
    pattern: PatternRule,
    #[serde(default = "default_required")]
    required: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum PatternRule {
    #[serde(rename = "MinLength")]
    MinLength { value: usize },
    #[serde(rename = "LengthRange")]
    LengthRange { min: usize, max: usize },
    #[serde(rename = "OneOf")]
    OneOf { value: Vec<String> },
    #[serde(rename = "NonEmpty")]
    NonEmpty,
    #[serde(rename = "Exact")]
    Exact { value: String },
    #[serde(rename = "Regex")]
    Regex { value: String },
}

fn default_required() -> bool {
    false
}

impl MetaTagRule {
    fn _matches_element(&self, element: &NodeData) -> bool {
        if let NodeData::Element { attrs, .. } = element {
            let attrs = attrs.borrow();
            if let Some(name) = &self.name {
                if attrs
                    .iter()
                    .any(|attr| attr.name.local.as_ref() == "name" && attr.value.as_ref() == name)
                {
                    return true;
                }
            }

            if let Some(property) = &self.property {
                if attrs.iter().any(|attr| {
                    attr.name.local.as_ref() == "property" && attr.value.as_ref() == property
                }) {
                    return true;
                }
            }
        }
        false
    }

    fn _validate_content(&self, content: &str) -> bool {
        match &self.pattern {
            PatternRule::MinLength { value } => content.len() >= *value,
            PatternRule::OneOf { value } => value.contains(&content.to_string()),
            PatternRule::NonEmpty => !content.is_empty(),
            PatternRule::Exact { value } => content == value,
            PatternRule::LengthRange { min, max } => content.len() >= *min && content.len() <= *max,
            PatternRule::Regex { value } => {
                let regex = Regex::new(value).unwrap();
                regex.is_match(content)
            }
        }
    }
}

impl HtmlLinter {
    pub(crate) fn check_text_content(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        match rule.condition.as_str() {
            "max-length" => {
                let max_length = rule
                    .options
                    .get("max_length")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(80);

                for node_idx in matches {
                    if let Some(node) = index.get_node(node_idx) {
                        let text = dom::utils::get_node_text_content(node_idx, index);
                        if text.len() > max_length {
                            results.push(self.create_lint_result(rule, node, index));
                        }
                    }
                }
            }
            "content-length" => {
                let min_length = rule
                    .options
                    .get("min_length")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
                let max_length = rule
                    .options
                    .get("max_length")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(usize::MAX);

                for node_idx in matches {
                    if let Some(node) = index.get_node(node_idx) {
                        let text = dom::utils::get_node_text_content(node_idx, index);
                        if text.len() < min_length || text.len() > max_length {
                            results.push(self.create_lint_result(rule, node, index));
                        }
                    }
                }
            }
            _ => {
                if let Some(pattern) = rule.options.get("pattern") {
                    let regex =
                        Regex::new(pattern).map_err(|e| LinterError::RuleError(e.to_string()))?;

                    for node_idx in matches {
                        if let Some(node) = index.get_node(node_idx) {
                            let mut text = String::new();
                            dom::utils::extract_text(node.handle.as_ref().unwrap(), &mut text);
                            let check_mode = rule
                                .options
                                .get("check_mode")
                                .map(String::as_str)
                                .unwrap_or("normal");

                            let matches = regex.is_match(&text);
                            let should_report = match check_mode {
                                "ensure_existence" => !matches,
                                "ensure_nonexistence" => matches,
                                _ => matches,
                            };

                            if should_report {
                                results.push(self.create_lint_result(rule, node, index));
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub(crate) fn check_element_content(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let should_report = match rule.condition.as_str() {
                    "meta-tags" => {
                        if let Some(required_tags) = rule.options.get("required_meta_tags") {
                            let meta_rules: Vec<MetaTagRule> = serde_json::from_str(required_tags)
                                .map_err(|e| LinterError::RuleError(e.to_string()))?;
                            !self.validate_meta_tags(node_idx, &meta_rules, index)?
                        } else {
                            false
                        }
                    }
                    "empty-or-default" => {
                        let content = dom::utils::get_node_text_content(node_idx, index);
                        content.is_empty()
                            || content.trim() == "Untitled"
                            || content.trim() == "Default"
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

    pub(crate) fn check_whitespace(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();

        match rule.condition.as_str() {
            "trailing-whitespace" => {
                let matches = index.query(&rule.selector);
                for node_idx in matches {
                    if let Some(node) = index.get_node(node_idx) {
                        let lines = node.source_info.source.lines();
                        for (i, line) in lines.enumerate() {
                            if line.trim_end().len() != line.len() {
                                results.push(LintResult {
                                    rule: rule.name.clone(),
                                    severity: rule.severity.clone(),
                                    message: "Line contains trailing whitespace".to_string(),
                                    location: Location {
                                        line: node.source_info.line + i,
                                        column: line.trim_end().len() + 1,
                                        element: index
                                            .resolve_symbol(node.tag_name)
                                            .unwrap_or_default()
                                            .to_string(),
                                    },
                                    source: line.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(results)
    }

    fn validate_meta_tags(
        &self,
        node_idx: usize,
        meta_rules: &[MetaTagRule],
        index: &DOMIndex,
    ) -> Result<bool, LinterError> {
        if let Some(_node) = index.get_node(node_idx) {
            for rule in meta_rules {
                let meta_nodes = index.query("meta");
                let mut found_valid_tag = false;

                for meta_node_idx in meta_nodes {
                    if let Some(meta_node) = index.get_node(meta_node_idx) {
                        let matches_identifier = meta_node.attributes.iter().any(|attr| {
                            let attr_name = index.resolve_symbol(attr.name).unwrap_or_default();
                            let attr_value = index.resolve_symbol(attr.value).unwrap_or_default();

                            match (&rule.name, &rule.property) {
                                (Some(name), _) => {
                                    attr_name == "name" && attr_value == name.to_string()
                                }
                                (_, Some(property)) => {
                                    attr_name == "property" && attr_value == property.to_string()
                                }
                                (None, None) => false,
                            }
                        });

                        if matches_identifier {
                            let content_valid = meta_node.attributes.iter().any(|attr| {
                                let name = index.resolve_symbol(attr.name).unwrap_or_default();
                                let value = index.resolve_symbol(attr.value).unwrap_or_default();
                                if name == "content" {
                                    let content = value.as_str();
                                    match &rule.pattern {
                                        PatternRule::MinLength { value: min_len } => {
                                            content.len() >= *min_len
                                        }
                                        PatternRule::LengthRange { min, max } => {
                                            content.len() >= *min && content.len() <= *max
                                        }
                                        PatternRule::OneOf { value } => {
                                            value.contains(&content.to_string())
                                        }
                                        PatternRule::NonEmpty => !content.trim().is_empty(),
                                        PatternRule::Exact { value: exact } => content == *exact,
                                        PatternRule::Regex { value: regex } => {
                                            let regex = Regex::new(regex).unwrap();
                                            regex.is_match(content)
                                        }
                                    }
                                } else {
                                    false
                                }
                            });

                            if content_valid {
                                found_valid_tag = true;
                                break;
                            }
                        }
                    }
                }

                if !found_valid_tag && rule.required {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
