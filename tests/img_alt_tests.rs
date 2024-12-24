use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use std::collections::HashMap;

fn create_img_alt_linter() -> HtmlLinter {
    let rules = vec![Rule {
        name: "img-alt".to_string(),
        rule_type: RuleType::AttributePresence,
        severity: Severity::Error,
        selector: "img".to_string(),
        condition: "alt-missing".to_string(),
        message: "Images must have alt attributes".to_string(),
        options: HashMap::new(),
    }];

    HtmlLinter::new(rules, None)
}

#[test]
fn test_img_missing_alt() {
    let linter = create_img_alt_linter();
    let html = r#"<html><body><img src="test.jpg"></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule, "img-alt");
    assert_eq!(results[0].severity, Severity::Error);
}

#[test]
fn test_img_with_alt() {
    let linter = create_img_alt_linter();
    let html = r#"<html><body><img src="test.jpg" alt="Test image"></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);
}
