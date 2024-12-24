use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use std::collections::HashMap;

fn create_inline_styles_linter() -> HtmlLinter {
    let rules = vec![Rule {
        name: "no-inline-styles".to_string(),
        rule_type: RuleType::AttributePresence,
        severity: Severity::Warning,
        selector: "*".to_string(),
        condition: "style-attribute".to_string(),
        message: "Inline styles should be avoided".to_string(),
        options: HashMap::new(),
    }];

    HtmlLinter::new(rules, None)
}

#[test]
fn test_element_with_inline_style() {
    let linter = create_inline_styles_linter();
    let html = r#"<div style="color: red;">Test</div>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule, "no-inline-styles");
    assert_eq!(results[0].severity, Severity::Warning);
}

#[test]
fn test_element_without_inline_style() {
    let linter = create_inline_styles_linter();
    let html = r#"<div class="red">Test</div>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);
}
