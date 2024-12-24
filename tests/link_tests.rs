use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use std::collections::HashMap;

fn setup_link_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "link-text".to_string(),
            rule_type: RuleType::TextContent,
            severity: Severity::Warning,
            selector: "a".to_string(),
            condition: "descriptive-text".to_string(),
            message: "Link text should be descriptive (avoid 'click here', 'learn more', etc.)"
                .to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "pattern".to_string(),
                    r#"^(click here|learn more|read more|more info)$"#.to_string(),
                );
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options
            },
        },
        Rule {
            name: "link-target".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "a[target='_blank']".to_string(),
            condition: "security-rel".to_string(),
            message: "Links opening in new tabs should have rel='noopener noreferrer'".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), r#"noopener noreferrer"#.to_string());
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("attributes".to_string(), "rel".to_string());
                options
            },
        },
        Rule {
            name: "link-href-javascript".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Error,
            selector: "a".to_string(),
            condition: "valid-href".to_string(),
            message: "Links should have a valid href attribute".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), r#"^javascript:"#.to_string());
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options.insert("attributes".to_string(), "href".to_string());
                options
            },
        },
        Rule {
            name: "link-href".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Error,
            selector: "a".to_string(),
            condition: "valid-href".to_string(),
            message: "Links should have a valid href attribute".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), '.'.to_string());
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("attributes".to_string(), "href".to_string());
                options
            },
        },
        Rule {
            name: "link-underline".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "a".to_string(),
            condition: "text-decoration".to_string(),
            message: "Links should be visually distinct (underlined by default)".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options.insert("attributes".to_string(), "style".to_string());
                options.insert(
                    "pattern".to_string(),
                    r#"text-decoration:\s*none"#.to_string(),
                );
                options
            },
        },
    ]
}

#[test]
fn test_link_with_all_best_practices() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com" target="_blank" rel="noopener noreferrer">Visit our documentation</a>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0, "Expected no violations for good link");
}

#[test]
fn test_link_with_generic_text() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com">click here</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-text"),
        "Should warn about generic link text"
    );
}

#[test]
fn test_blank_target_without_rel() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com" target="_blank">Documentation</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-target"),
        "Should warn about missing rel attribute on _blank target"
    );
}

#[test]
fn test_link_with_javascript_href() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="javascript:void(0)">Invalid href</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-href-javascript"),
        "Should error on javascript: href values"
    );
}

#[test]
fn test_link_without_href() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a>Missing href</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-href"),
        "Should error on missing href attribute"
    );
}

#[test]
fn test_link_with_text_decoration_none() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com" style="text-decoration: none">Hidden link</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-underline"),
        "Should warn about removing link underline"
    );
}

#[test]
fn test_link_with_learn_more_text() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com">learn more</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-text"),
        "Should warn about non-descriptive 'learn more' text"
    );
}

#[test]
fn test_link_with_read_more_text() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com">read more</a>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "link-text"),
        "Should warn about non-descriptive 'read more' text"
    );
}

#[test]
fn test_link_with_proper_rel_no_target() {
    let linter = HtmlLinter::new(setup_link_rules(), None);
    let html = r#"<a href="https://example.com" rel="noopener noreferrer">Safe link</a>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Should not require rel attributes when target=_blank is not present"
    );
}
