use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::RcDom;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

mod checks;
mod dom;

use dom::{DOMIndex, IndexedNode};

#[derive(Error, Debug)]
pub enum LinterError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Rule error: {0}")]
    RuleError(String),
    #[error("Invalid selector: {0}")]
    SelectorError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RuleType {
    ElementPresence,
    AttributePresence,
    AttributeValue,
    ElementOrder,
    TextContent,
    ElementContent,
    WhiteSpace,
    Nesting,
    Semantics,
    Compound,
    Custom(String),
    DocumentStructure,
    ElementCount,
    ElementCase,
    AttributeQuotes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub selector: String,  // CSS-like selector
    pub condition: String, // Rule-specific condition
    pub message: String,   // Error message
    #[serde(default)]
    pub options: HashMap<String, String>, // Additional rule options
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct LintResult {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    pub location: Location,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub element: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LinterOptions {
    pub ignore_files: Vec<String>,
    pub custom_selectors: HashMap<String, String>,
    pub max_line_length: Option<usize>,
    pub allow_inline_styles: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetaTagRule {
    name: Option<String>,     // name attribute
    property: Option<String>, // property attribute (for Open Graph etc.)
    pattern: MetaTagPattern,  // pattern to match against
    required: bool,           // whether this meta tag is required
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum MetaTagPattern {
    Regex(String),      // Regular expression pattern
    MinLength(usize),   // Minimum content length
    MaxLength(usize),   // Maximum content length
    NonEmpty,           // Must not be empty
    Exact(String),      // Exact match
    OneOf(Vec<String>), // Must match one of these values
    Contains(String),   // Must contain this string
    StartsWith(String), // Must start with this string
    EndsWith(String),   // Must end with this string
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CompoundCondition {
    TextContent {
        pattern: String,
    },
    AttributeValue {
        attribute: String,
        pattern: String,
    },
    AttributeReference {
        attribute: String,
        reference_must_exist: bool,
    },
    ElementPresence {
        selector: String,
    },
}

pub struct HtmlLinter {
    pub(crate) rules: Vec<Rule>,
    options: LinterOptions,
}

impl HtmlLinter {
    pub fn new(rules: Vec<Rule>, options: Option<LinterOptions>) -> Self {
        Self {
            rules,
            options: options.unwrap_or_default(),
        }
    }

    pub fn lint(&self, html: &str) -> Result<Vec<LintResult>, LinterError> {
        let dom = parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map_err(|e| LinterError::ParseError(e.to_string()))?;

        let index = DOMIndex::new(&dom, html);
        let mut results = Vec::new();

        // Process rules in parallel using rayon
        for rule in &self.rules {
            if !self.should_ignore_rule(&rule.name) {
                results.extend(self.process_rule(rule, &index)?);
            }
        }

        Ok(results)
    }

    pub fn from_json(json: &str, options: Option<LinterOptions>) -> Result<Self, LinterError> {
        let rules: Vec<Rule> = serde_json::from_str(json)
            .map_err(|e| LinterError::ParseError(format!("Failed to parse rules JSON: {}", e)))?;
        Ok(Self::new(rules, options))
    }

    pub fn from_json_file(path: &str, options: Option<LinterOptions>) -> Result<Self, LinterError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content, options)
    }

    fn should_ignore_rule(&self, rule_name: &str) -> bool {
        self.options.ignore_files.iter().any(|pattern| {
            if let Ok(regex) = Regex::new(pattern) {
                regex.is_match(rule_name)
            } else {
                pattern == rule_name
            }
        })
    }

    fn process_rule(&self, rule: &Rule, index: &DOMIndex) -> Result<Vec<LintResult>, LinterError> {
        match rule.rule_type {
            RuleType::ElementPresence => self.check_element_presence(rule, index),
            RuleType::AttributePresence => self.check_attribute_presence(rule, index),
            RuleType::AttributeValue => self.check_attribute_value(rule, index),
            RuleType::ElementOrder => self.check_element_order(rule, index),
            RuleType::TextContent => self.check_text_content(rule, index),
            RuleType::ElementContent => self.check_element_content(rule, index),
            RuleType::WhiteSpace => self.check_whitespace(rule, index),
            RuleType::Nesting => self.check_nesting(rule, index),
            RuleType::Semantics => self.check_semantics(rule, index),
            RuleType::Compound => self.check_compound(rule, index),
            RuleType::Custom(ref validator) => self.check_custom(rule, validator, index),
            RuleType::DocumentStructure => self.check_document_structure(rule, index),
            RuleType::ElementCount => self.check_element_count(rule, index),
            RuleType::ElementCase => self.check_element_case(rule, index),
            RuleType::AttributeQuotes => self.check_attribute_quotes(rule, index),
        }
    }

    fn create_lint_result(&self, rule: &Rule, node: &IndexedNode, index: &DOMIndex) -> LintResult {
        LintResult {
            rule: rule.name.clone(),
            severity: rule.severity.clone(),
            message: rule.message.clone(),
            location: Location {
                line: node.source_info.line,
                column: node.source_info.column,
                element: index
                    .resolve_symbol(node.tag_name)
                    .unwrap_or_default()
                    .to_string(),
            },
            source: node.source_info.source.clone(),
        }
    }

    pub fn get_rules(&self) -> Vec<Rule> {
        self.rules.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_linting() {
        let rules = vec![Rule {
            name: "img-alt".to_string(),
            rule_type: RuleType::AttributePresence,
            severity: Severity::Error,
            selector: "img".to_string(),
            condition: "alt-missing".to_string(),
            message: "Image must have alt attribute".to_string(),
            options: HashMap::new(),
        }];

        let linter = HtmlLinter::new(rules, None);
        let html = r#"<img src="test.jpg">"#;
        let results = linter.lint(html).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, Severity::Error);
    }

    #[test]
    fn test_compound_rule() {
        // Add more comprehensive tests
    }
}
