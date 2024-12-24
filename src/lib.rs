use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::default::Default;
use thiserror::Error;

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
    ElementPresence,   // Check if elements exist/don't exist
    AttributePresence, // Check if attributes exist/don't exist
    AttributeValue,    // Validate attribute values
    ElementOrder,      // Check element ordering
    TextContent,       // Add this new variant
    ElementContent,    // Check element content
    WhiteSpace,        // Check whitespace/formatting
    Nesting,           // Check element nesting
    Semantics,         // Check semantic rules
    Custom(String),    // Custom rule type with validation function
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

pub struct HtmlLinter {
    pub(crate) rules: Vec<Rule>,
    options: LinterOptions,
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

impl HtmlLinter {
    pub fn new(rules: Vec<Rule>, options: Option<LinterOptions>) -> Self {
        Self {
            rules,
            options: options.unwrap_or_default(),
        }
    }

    pub fn lint(&self, html: &str) -> Result<Vec<LintResult>, LinterError> {
        let dom = self.parse_html(html)?;
        let mut results = Vec::new();
        let source_lines: Vec<&str> = html.lines().collect();

        for rule in &self.rules {
            match rule.rule_type {
                RuleType::ElementPresence => {
                    self.check_element_presence(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::AttributePresence => {
                    self.check_attribute_presence(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::AttributeValue => {
                    self.check_attribute_value(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::ElementOrder => {
                    self.check_element_order(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::TextContent => {
                    self.check_text_content(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::ElementContent => {
                    self.check_element_content(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::WhiteSpace => {
                    self.check_whitespace(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::Nesting => self.check_nesting(&dom, rule, &mut results, &source_lines)?,
                RuleType::Semantics => {
                    self.check_semantics(&dom, rule, &mut results, &source_lines)?
                }
                RuleType::Custom(ref validator) => {
                    self.check_custom(&dom, rule, validator, &mut results, &source_lines)?
                }
            }
        }

        Ok(results)
    }

    fn parse_html(&self, html: &str) -> Result<RcDom, LinterError> {
        parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map_err(|e| LinterError::ParseError(e.to_string()))
    }

    fn check_element_presence(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        for element in elements {
            let should_report = match rule.condition.as_str() {
                "required" => false, // Element exists, which is good
                "forbidden" => true, // Element exists but shouldn't
                "semantic-alternative-available" => self.check_semantic_alternative(&element),
                _ => false,
            };

            if should_report {
                if let Some(location) = self.get_node_location(&element, source_lines) {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: rule.message.clone(),
                        location,
                        source: self.get_element_source(&element),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_attribute_presence(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        for element in elements {
            if let NodeData::Element { ref attrs, .. } = element.data {
                let attrs = attrs.borrow();
                let attr_name = rule.condition.split('-').next().unwrap_or("");

                let has_attribute = attrs
                    .iter()
                    .any(|attr| attr.name.local.to_string() == attr_name);

                let should_report = match rule.condition.as_str() {
                    "alt-missing" => !has_attribute,
                    "style-attribute" if !self.options.allow_inline_styles => has_attribute,
                    _ => false,
                };

                if should_report {
                    if let Some(location) = self.get_node_location(&element, source_lines) {
                        results.push(LintResult {
                            rule: rule.name.clone(),
                            severity: rule.severity.clone(),
                            message: rule.message.clone(),
                            location,
                            source: self.get_element_source(&element),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn check_attribute_value(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        let target_attributes = rule
            .options
            .get("attributes")
            .map(|attrs| {
                attrs
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["*".to_string()]);

        for element in elements {
            if let NodeData::Element { ref attrs, .. } = element.data {
                let attrs = attrs.borrow();
                let mut problematic_attrs = Vec::new();

                let check_mode = rule
                    .options
                    .get("check_mode")
                    .map(|s| s.as_str())
                    .unwrap_or("normal");

                if let Some(pattern) = rule.options.get("pattern") {
                    let regex =
                        Regex::new(pattern).map_err(|e| LinterError::RuleError(e.to_string()))?;

                    let mut all_attributes_match = true;
                    let mut found_any_target = false;

                    // First, check if any of the target attributes exist
                    for attr in attrs.iter() {
                        if target_attributes.contains(&"*".to_string())
                            || target_attributes.contains(&attr.name.local.to_string())
                        {
                            found_any_target = true;
                            break;
                        }
                    }

                    // If we're in ensure_existence mode and no target attributes were found, report it
                    if check_mode == "ensure_existence" && !found_any_target {
                        if let Some(location) = self.get_node_location(&element, source_lines) {
                            results.push(LintResult {
                                rule: rule.name.clone(),
                                severity: rule.severity.clone(),
                                message: format!(
                                    "{} (missing required attributes: {})",
                                    rule.message,
                                    target_attributes.join(", ")
                                ),
                                location,
                                source: self.get_element_source(&element),
                            });
                        }
                        continue;
                    }

                    // Rest of the existing attribute checking logic
                    for attr in attrs.iter() {
                        if !target_attributes.contains(&"*".to_string())
                            && !target_attributes.contains(&attr.name.local.to_string())
                        {
                            continue;
                        }

                        found_any_target = true;
                        let matches = regex.is_match(&attr.value);

                        match check_mode {
                            "ensure_existence" => {
                                if !matches
                                    && (target_attributes.contains(&"*".to_string())
                                        || target_attributes.contains(&attr.name.local.to_string()))
                                {
                                    all_attributes_match = false;
                                    problematic_attrs.push((
                                        attr.name.local.to_string(),
                                        attr.value.to_string(),
                                    ));
                                }
                            }
                            "ensure_nonexistence" => {
                                if matches {
                                    problematic_attrs.push((
                                        attr.name.local.to_string(),
                                        attr.value.to_string(),
                                    ));
                                }
                            }
                            _ => {
                                // Normal mode - report matching attributes
                                if matches {
                                    problematic_attrs.push((
                                        attr.name.local.to_string(),
                                        attr.value.to_string(),
                                    ));
                                }
                            }
                        }
                    }

                    let should_report = match check_mode {
                        "check_existence" => found_any_target && !all_attributes_match,
                        "ensure_nonexistence" => !problematic_attrs.is_empty(),
                        _ => !problematic_attrs.is_empty(),
                    };

                    if should_report {
                        if let Some(location) = self.get_node_location(&element, source_lines) {
                            let attr_list = problematic_attrs
                                .iter()
                                .map(|(name, value)| format!("{}=\"{}\"", name, value))
                                .collect::<Vec<_>>()
                                .join(", ");

                            results.push(LintResult {
                                rule: rule.name.clone(),
                                severity: rule.severity.clone(),
                                message: format!("{} (attributes: {})", rule.message, attr_list),
                                location,
                                source: self.get_element_source(&element),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn check_element_order(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        if rule.condition == "sequential-order" {
            let mut current_level = 0;
            let mut last_heading: Option<Handle> = None;

            self.walk_dom(&dom.document, |node| {
                if let NodeData::Element { ref name, .. } = node.data {
                    if name.local.to_string().starts_with('h') {
                        let level = name.local.to_string()[1..].parse::<i32>().unwrap_or(0);

                        if let Some(last) = &last_heading {
                            if let NodeData::Element { ref name, .. } = last.data {
                                let last_level =
                                    name.local.to_string()[1..].parse::<i32>().unwrap_or(0);
                                if level > last_level + 1 {
                                    if let Some(location) =
                                        self.get_node_location(node, source_lines)
                                    {
                                        results.push(LintResult {
                                            rule: rule.name.clone(),
                                            severity: rule.severity.clone(),
                                            message: format!(
                                                "Heading level jumped from h{} to h{}",
                                                last_level, level
                                            ),
                                            location,
                                            source: self.get_element_source(node),
                                        });
                                    }
                                }
                            }
                        }
                        current_level = level;
                        last_heading = Some(node.clone());
                    }
                }
            });
        }

        Ok(())
    }

    fn check_nesting(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        match rule.condition.as_str() {
            "parent-label-or-for" => {
                let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

                for element in elements {
                    let has_label_parent = self.has_parent_of_type(&element, "label");
                    let has_for_attribute =
                        if let NodeData::Element { ref attrs, .. } = element.data {
                            let attrs = attrs.borrow();
                            attrs.iter().any(|attr| attr.name.local.to_string() == "id")
                                && self.has_matching_label(&dom.document, &attrs)
                        } else {
                            false
                        };

                    if !has_label_parent && !has_for_attribute {
                        if let Some(location) = self.get_node_location(&element, source_lines) {
                            results.push(LintResult {
                                rule: rule.name.clone(),
                                severity: rule.severity.clone(),
                                message: rule.message.clone(),
                                location,
                                source: self.get_element_source(&element),
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn check_whitespace(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let _ = dom;
        // Check for consistent indentation and spacing
        if let Some(max_length) = self.options.max_line_length {
            for (line_num, line) in source_lines.iter().enumerate() {
                if line.len() > max_length {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: format!(
                            "Line exceeds maximum length of {} characters",
                            max_length
                        ),
                        location: Location {
                            line: line_num + 1,
                            column: max_length + 1,
                            element: line.to_string(),
                        },
                        source: line.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_semantics(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        for element in elements {
            let semantic_issue = match rule.condition.as_str() {
                "semantic-heading" => self.check_semantic_heading(&element),
                "semantic-list" => self.check_semantic_list(&element),
                "semantic-structure" => self.check_semantic_structure(&element),
                _ => false,
            };

            if semantic_issue {
                if let Some(location) = self.get_node_location(&element, source_lines) {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: rule.message.clone(),
                        location,
                        source: self.get_element_source(&element),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_custom(
        &self,
        dom: &RcDom,
        rule: &Rule,
        validator: &str,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        for element in elements {
            let custom_issue = match validator {
                "no-empty-links" => {
                    if let NodeData::Element { ref name, .. } = element.data {
                        if name.local.to_string() == "a" {
                            element.children.borrow().is_empty()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "no-empty-headings" => {
                    if let NodeData::Element { ref name, .. } = element.data {
                        if name.local.to_string().starts_with('h') {
                            element.children.borrow().is_empty()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if custom_issue {
                if let Some(location) = self.get_node_location(&element, source_lines) {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: rule.message.clone(),
                        location,
                        source: self.get_element_source(&element),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_element_content(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        for element in elements {
            let should_report = match rule.condition.as_str() {
                "empty-or-default" => {
                    let content = self.get_element_content(&element);
                    content.is_empty()
                        || content.trim() == "Untitled"
                        || content.trim() == "Default"
                }
                "missing-meta-description" => {
                    if let NodeData::Element { ref attrs, .. } = element.data {
                        let attrs = attrs.borrow();
                        // Use meta_tag_matches for more robust meta tag checking
                        !self.meta_tag_matches(
                            &attrs,
                            &MetaTagRule {
                                name: Some("description".to_string()),
                                property: None,
                                pattern: MetaTagPattern::NonEmpty,
                                required: true,
                            },
                        )
                    } else {
                        false
                    }
                }
                "meta-tags" => {
                    if let Some(rules_str) = rule.options.get("required_meta_tags") {
                        if let Ok(required_rules) =
                            serde_json::from_str::<Vec<MetaTagRule>>(rules_str)
                        {
                            let missing = self.check_required_meta_tags(&element, &required_rules);
                            if !missing.is_empty() {
                                // Add each missing meta tag as a separate result
                                for msg in missing {
                                    if let Some(location) =
                                        self.get_node_location(&element, source_lines)
                                    {
                                        results.push(LintResult {
                                            rule: rule.name.clone(),
                                            severity: rule.severity.clone(),
                                            message: msg,
                                            location,
                                            source: self.get_element_source(&element),
                                        });
                                    }
                                }
                            }
                            false // We've already added results directly
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if should_report {
                if let Some(location) = self.get_node_location(&element, source_lines) {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: rule.message.clone(),
                        location,
                        source: self.get_element_source(&element),
                    });
                }
            }
        }

        Ok(())
    }

    fn get_element_content(&self, node: &Handle) -> String {
        let mut content = String::new();
        for child in node.children.borrow().iter() {
            match child.data {
                NodeData::Text { ref contents } => {
                    content.push_str(&contents.borrow());
                }
                NodeData::Element { .. } => {
                    content.push_str(&self.get_element_content(child));
                }
                _ => {}
            }
        }
        content
    }

    // Helper methods

    fn get_elements_by_selector(
        &self,
        handle: &Handle,
        selector: &str,
    ) -> Result<Vec<Handle>, LinterError> {
        let mut elements = Vec::new();
        let selector = selector.trim();

        self.walk_dom(handle, |node| {
            if let NodeData::Element { .. } = node.data {
                if selector == "*" || {
                    if let NodeData::Element { ref name, .. } = node.data {
                        self.matches_selector(&name.local.to_string(), selector)
                    } else {
                        false
                    }
                } {
                    elements.push(node.clone());
                }
            }
        });

        Ok(elements)
    }
    fn matches_selector(&self, element_name: &str, selector: &str) -> bool {
        // Handle multiple selectors separated by commas
        if selector.contains(',') {
            return selector
                .split(',')
                .map(|s| s.trim())
                .any(|s| self.matches_single_selector(element_name, s));
        }

        self.matches_single_selector(element_name, selector)
    }

    fn matches_single_selector(&self, element_name: &str, selector: &str) -> bool {
        // Handle empty selector
        if selector.is_empty() {
            return false;
        }

        // Handle group selectors (comma-separated)
        if selector.contains(',') {
            return selector
                .split(',')
                .map(str::trim)
                .any(|s| self.matches_single_selector(element_name, s));
        }

        // Since we only have element_name, we can only match:
        // 1. Type selectors (div, span, etc.)
        // 2. Universal selector (*)
        // 3. The element part of complex selectors

        // Extract the element/type part of the selector
        let element_part = if selector.contains(|c| c == '.' || c == '#' || c == '[' || c == ':') {
            // For complex selectors, get the element part before any modifier
            selector
                .chars()
                .take_while(|&c| c.is_ascii_alphabetic() || c == '-' || c == '_' || c == '*')
                .collect::<String>()
        } else {
            // Simple element selector
            selector.to_string()
        };

        // Handle universal selector
        if element_part == "*" {
            return true;
        }

        // If element part is empty (selector starts with ., #, [, or :)
        // then any element name is valid
        if element_part.is_empty() {
            return true;
        }

        // Match element name exactly
        element_name == element_part
    }
    fn walk_dom<F>(&self, handle: &Handle, mut callback: F)
    where
        F: FnMut(&Handle),
    {
        let mut stack = vec![handle.clone()];

        while let Some(current) = stack.pop() {
            callback(&current);

            for child in current.children.borrow().iter().rev() {
                stack.push(child.clone());
            }
        }
    }

    fn get_node_location(&self, node: &Handle, source_lines: &[&str]) -> Option<Location> {
        let _ = source_lines;
        // This is a simplified implementation
        // In a real implementation, you would track line/column during parsing
        Some(Location {
            line: 1,
            column: 1,
            element: self.get_element_name(node),
        })
    }

    fn get_element_name(&self, node: &Handle) -> String {
        if let NodeData::Element { ref name, .. } = node.data {
            name.local.to_string()
        } else {
            String::new()
        }
    }

    fn get_element_source(&self, node: &Handle) -> String {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = node.data
        {
            let attrs_str: String = attrs
                .borrow()
                .iter()
                .map(|attr| format!(" {}=\"{}\"", attr.name.local, attr.value))
                .collect();
            format!("<{}{}>", name.local, attrs_str)
        } else {
            String::new()
        }
    }

    fn has_parent_of_type(&self, node: &Handle, parent_type: &str) -> bool {
        let mut current_weak = match node.parent.take() {
            Some(weak) => weak,
            None => return false,
        };
        // Put back the original parent
        node.parent.set(Some(current_weak.clone()));

        while let Some(current) = current_weak.upgrade() {
            if let NodeData::Element { ref name, .. } = current.data {
                if name.local.to_string() == parent_type {
                    return true;
                }
            }

            // Get next parent before current one is dropped
            match current.parent.take() {
                Some(next_weak) => {
                    current.parent.set(Some(next_weak.clone()));
                    current_weak = next_weak;
                }
                None => break,
            }
        }
        false
    }

    fn has_matching_label(
        &self,
        document: &Handle,
        attrs: &[html5ever::interface::Attribute],
    ) -> bool {
        let id = attrs
            .iter()
            .find(|attr| attr.name.local.to_string() == "id")
            .map(|attr| attr.value.to_string());

        if let Some(id) = id {
            let mut has_matching_label = false;
            self.walk_dom(document, |node| {
                if let NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } = node.data
                {
                    if name.local.to_string() == "label" {
                        let attrs = attrs.borrow();
                        if attrs.iter().any(|attr| {
                            attr.name.local.to_string() == "for" && attr.value.to_string() == id
                        }) {
                            has_matching_label = true;
                        }
                    }
                }
            });
            has_matching_label
        } else {
            false
        }
    }

    fn check_semantic_heading(&self, node: &Handle) -> bool {
        // Check if non-heading elements are being used as headings
        if let NodeData::Element { ref name, .. } = node.data {
            !name.local.to_string().starts_with('h') && self.appears_to_be_heading(node)
        } else {
            false
        }
    }

    fn check_semantic_list(&self, node: &Handle) -> bool {
        // Check if list-like content isn't using proper list elements
        if let NodeData::Element { ref name, .. } = node.data {
            !["ul", "ol", "dl"].contains(&name.local.to_string().as_str())
                && self.appears_to_be_list(node)
        } else {
            false
        }
    }

    fn check_semantic_structure(&self, node: &Handle) -> bool {
        // Check for proper semantic structure usage
        if let NodeData::Element { ref name, .. } = node.data {
            match name.local.to_string().as_str() {
                "div" | "span" => self.should_use_semantic_element(node),
                _ => false,
            }
        } else {
            false
        }
    }

    fn check_semantic_alternative(&self, node: &Handle) -> bool {
        // Check if there's a better semantic element available
        if let NodeData::Element { ref name, .. } = node.data {
            match name.local.to_string().as_str() {
                "div" | "span" => self.has_better_semantic_alternative(node),
                _ => false,
            }
        } else {
            false
        }
    }

    // Helper methods for semantic checks
    fn appears_to_be_heading(&self, node: &Handle) -> bool {
        // Simplified check - could be enhanced with more sophisticated detection
        if let NodeData::Element { ref attrs, .. } = node.data {
            let attrs = attrs.borrow();
            attrs.iter().any(|attr| {
                attr.name.local.to_string() == "class" && attr.value.to_string().contains("heading")
            })
        } else {
            false
        }
    }

    fn appears_to_be_list(&self, node: &Handle) -> bool {
        // Simplified check for list-like structures
        let children = node.children.borrow();
        children.len() > 2
            && children.iter().all(|child| {
                if let NodeData::Element { ref name, .. } = child.data {
                    name.local.to_string() == "div" || name.local.to_string() == "p"
                } else {
                    false
                }
            })
    }

    fn should_use_semantic_element(&self, node: &Handle) -> bool {
        // Check if a div/span should be replaced with a semantic element
        if let NodeData::Element { ref attrs, .. } = node.data {
            let attrs = attrs.borrow();
            attrs.iter().any(|attr| {
                attr.name.local.to_string() == "class"
                    && ["header", "footer", "nav", "main", "article", "section"]
                        .iter()
                        .any(|semantic| attr.value.to_string().contains(semantic))
            })
        } else {
            false
        }
    }

    fn has_better_semantic_alternative(&self, node: &Handle) -> bool {
        // Check if there's a more appropriate semantic element
        if let NodeData::Element { ref attrs, .. } = node.data {
            let attrs = attrs.borrow();
            attrs.iter().any(|attr| {
                attr.name.local.to_string() == "role"
                    || (attr.name.local.to_string() == "class"
                        && ["button", "navigation", "content", "header"]
                            .iter()
                            .any(|semantic| attr.value.to_string().contains(semantic)))
            })
        } else {
            false
        }
    }

    fn check_required_meta_tags(&self, head: &Handle, required: &[MetaTagRule]) -> Vec<String> {
        let mut missing = Vec::new();

        for rule in required {
            let mut found = false;

            self.walk_dom(head, |node| {
                if let NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } = node.data
                {
                    if name.local.to_string() == "meta" {
                        let attrs = attrs.borrow();
                        let matches = self.meta_tag_matches(&attrs, rule);
                        if matches {
                            found = true;
                        }
                    }
                }
            });

            if !found && rule.required {
                let msg = match (&rule.name, &rule.property) {
                    (Some(name), _) => format!(
                        "Missing or invalid meta tag with name=\"{}\". Expected pattern: {}",
                        name,
                        self.pattern_description(&rule.pattern)
                    ),
                    (_, Some(prop)) => format!(
                        "Missing or invalid meta tag with property=\"{}\". Expected pattern: {}",
                        prop,
                        self.pattern_description(&rule.pattern)
                    ),
                    _ => "Missing required meta tag".to_string(),
                };
                missing.push(msg);
            }
        }

        missing
    }

    fn meta_tag_matches(
        &self,
        attrs: &[html5ever::interface::Attribute],
        rule: &MetaTagRule,
    ) -> bool {
        let mut name_or_property_matches = false;
        let mut content_matches = false;

        // Check if name/property matches
        if let Some(ref name) = rule.name {
            name_or_property_matches = attrs.iter().any(|attr| {
                attr.name.local.to_string() == "name" && attr.value.to_string() == *name
            });
        }

        if let Some(ref property) = rule.property {
            name_or_property_matches |= attrs.iter().any(|attr| {
                attr.name.local.to_string() == "property" && attr.value.to_string() == *property
            });
        }

        // If name/property matches, check content pattern
        if name_or_property_matches {
            if let Some(content_attr) = attrs
                .iter()
                .find(|attr| attr.name.local.to_string() == "content")
            {
                content_matches = self.content_matches_pattern(&content_attr.value, &rule.pattern);
            }
        }

        name_or_property_matches && content_matches
    }

    fn content_matches_pattern(&self, content: &str, pattern: &MetaTagPattern) -> bool {
        match pattern {
            MetaTagPattern::Regex(regex_str) => {
                if let Ok(regex) = Regex::new(regex_str) {
                    regex.is_match(content)
                } else {
                    false
                }
            }
            MetaTagPattern::MinLength(min) => content.len() >= *min,
            MetaTagPattern::MaxLength(max) => content.len() <= *max,
            MetaTagPattern::NonEmpty => !content.trim().is_empty(),
            MetaTagPattern::Exact(expected) => content == expected,
            MetaTagPattern::OneOf(values) => values.contains(&content.to_string()),
            MetaTagPattern::Contains(substring) => content.contains(substring),
            MetaTagPattern::StartsWith(prefix) => content.starts_with(prefix),
            MetaTagPattern::EndsWith(suffix) => content.ends_with(suffix),
        }
    }

    fn pattern_description(&self, pattern: &MetaTagPattern) -> String {
        match pattern {
            MetaTagPattern::Regex(regex) => format!("matches regex '{}'", regex),
            MetaTagPattern::MinLength(min) => format!("minimum length of {} characters", min),
            MetaTagPattern::MaxLength(max) => format!("maximum length of {} characters", max),
            MetaTagPattern::NonEmpty => "non-empty content".to_string(),
            MetaTagPattern::Exact(expected) => format!("exactly '{}'", expected),
            MetaTagPattern::OneOf(values) => format!("one of {:?}", values),
            MetaTagPattern::Contains(substring) => format!("contains '{}'", substring),
            MetaTagPattern::StartsWith(prefix) => format!("starts with '{}'", prefix),
            MetaTagPattern::EndsWith(suffix) => format!("ends with '{}'", suffix),
        }
    }

    fn check_text_content(
        &self,
        dom: &RcDom,
        rule: &Rule,
        results: &mut Vec<LintResult>,
        source_lines: &[&str],
    ) -> Result<(), LinterError> {
        let elements = self.get_elements_by_selector(&dom.document, &rule.selector)?;

        for element in elements {
            let text_content = self.get_element_content(&element);

            if let Some(pattern) = rule.options.get("pattern") {
                let regex =
                    Regex::new(pattern).map_err(|e| LinterError::RuleError(e.to_string()))?;

                let check_mode = rule
                    .options
                    .get("check_mode")
                    .map(|s| s.as_str())
                    .unwrap_or("normal");

                let matches = regex.is_match(&text_content);
                let should_report = match check_mode {
                    "ensure_existence" => !matches,
                    "ensure_nonexistence" => matches,
                    _ => matches,
                };

                if should_report {
                    if let Some(location) = self.get_node_location(&element, source_lines) {
                        results.push(LintResult {
                            rule: rule.name.clone(),
                            severity: rule.severity.clone(),
                            message: format!(
                                "{} (text content: \"{}\")",
                                rule.message,
                                text_content.trim()
                            ),
                            location,
                            source: self.get_element_source(&element),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub fn from_json(json: &str, options: Option<LinterOptions>) -> Result<Self, LinterError> {
        let rules: Vec<Rule> = serde_json::from_str(json)
            .map_err(|e| LinterError::ParseError(format!("Failed to parse rules JSON: {}", e)))?;

        Ok(Self {
            rules,
            options: options.unwrap_or_default(),
        })
    }

    pub fn from_json_file(path: &str, options: Option<LinterOptions>) -> Result<Self, LinterError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content, options)
    }

    pub fn get_rules(&self) -> &Vec<Rule> {
        &self.rules
    }
}
