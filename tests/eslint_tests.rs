use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use std::collections::HashMap;

fn setup_eslint_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "no-duplicate-attrs".to_string(),
            rule_type: RuleType::AttributePresence,
            severity: Severity::Error,
            selector: "*".to_string(),
            condition: "duplicate-attributes".to_string(),
            message: "Duplicate attributes are not allowed".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "no-duplicate-id".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Error,
            selector: "[id]".to_string(),
            condition: "unique-id".to_string(),
            message: "IDs must be unique".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "no-inline-styles".to_string(),
            rule_type: RuleType::AttributePresence,
            severity: Severity::Warning,
            selector: "*".to_string(),
            condition: "style-attribute".to_string(),
            message: "Inline styles should be avoided".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "require-doctype".to_string(),
            rule_type: RuleType::DocumentStructure,
            severity: Severity::Error,
            selector: "html".to_string(),
            condition: "doctype-present".to_string(),
            message: "HTML documents must have a DOCTYPE declaration".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "require-lang".to_string(),
            rule_type: RuleType::AttributePresence,
            severity: Severity::Error,
            selector: "html".to_string(),
            condition: "lang-attribute".to_string(),
            message: "The <html> element must have a lang attribute".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "no-extra-spacing-text".to_string(),
            rule_type: RuleType::TextContent,
            severity: Severity::Warning,
            selector: "*".to_string(),
            condition: "consecutive-spaces".to_string(),
            message: "Unnecessary consecutive spaces are not allowed".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), r#"\s{2,}"#.to_string());
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options
            },
        },
        Rule {
            name: "no-obsolete-tags".to_string(),
            rule_type: RuleType::ElementPresence,
            severity: Severity::Error,
            selector: "marquee, blink, font, center".to_string(),
            condition: "element-present".to_string(),
            message: "Obsolete HTML tags are not allowed".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "no-multiple-h1".to_string(),
            rule_type: RuleType::ElementCount,
            severity: Severity::Error,
            selector: "h1".to_string(),
            condition: "max-count".to_string(),
            message: "Only one <h1> element is allowed per page".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("max".to_string(), "1".to_string());
                options
            },
        },
        Rule {
            name: "require-meta-description".to_string(),
            rule_type: RuleType::ElementPresence,
            severity: Severity::Warning,
            selector: "head meta[name='description']".to_string(),
            condition: "element-present".to_string(),
            message: "Meta description is required".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "require-title".to_string(),
            rule_type: RuleType::ElementPresence,
            severity: Severity::Error,
            selector: "head title".to_string(),
            condition: "element-present".to_string(),
            message: "Title element is required in head".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "no-positive-tabindex".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "[tabindex]".to_string(),
            condition: "positive-number".to_string(),
            message: "Positive tabindex values should be avoided".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), r#"^[1-9]\d*$"#.to_string());
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options.insert("attributes".to_string(), "tabindex".to_string());
                options
            },
        },
        Rule {
            name: "require-img-alt".to_string(),
            rule_type: RuleType::AttributePresence,
            severity: Severity::Error,
            selector: "img".to_string(),
            condition: "alt-attribute".to_string(),
            message: "Images must have alt attributes".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "lowercase".to_string(),
            rule_type: RuleType::ElementCase,
            severity: Severity::Warning,
            selector: "*".to_string(),
            condition: "lowercase".to_string(),
            message: "HTML tags and attributes should be lowercase".to_string(),
            options: HashMap::new(),
        },
        Rule {
            name: "quotes".to_string(),
            rule_type: RuleType::AttributeQuotes,
            severity: Severity::Warning,
            selector: "*".to_string(),
            condition: "quote-style".to_string(),
            message: "Use double quotes for attribute values".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("style".to_string(), "double".to_string());
                options
            },
        },
    ]
}

#[test]
fn test_valid_html_document() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>Valid Document</title>
</head>
<body>
    <div id="unique">Content</div>
</body>
</html>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0, "Expected no violations for valid HTML");
}

#[test]
fn test_duplicate_attributes() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"<div class="foo" class="bar">Duplicate class</div>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "no-duplicate-attrs"),
        "Should detect duplicate attributes"
    );
}

#[test]
fn test_duplicate_ids() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"
        <div id="test">First</div>
        <div id="test">Second</div>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "no-duplicate-id"),
        "Should detect duplicate IDs"
    );
}

#[test]
fn test_inline_styles() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"<div style="color: red;">Styled content</div>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "no-inline-styles"),
        "Should warn about inline styles"
    );
}

#[test]
fn test_missing_doctype() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"
<html lang="en">
<head><title>No Doctype</title></head>
<body></body>
</html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "require-doctype"),
        "Should require DOCTYPE declaration"
    );
}

#[test]
fn test_missing_lang() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"<!DOCTYPE html>
<html>
<head><title>No Lang</title></head>
<body></body>
</html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "require-lang"),
        "Should require lang attribute on html element"
    );
}

#[test]
fn test_multiple_violations() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"<html>
<body>
    <div id="test" style="color: red;" class="foo" class="bar">
        <span id="test">Duplicate ID</span>
    </div>
</body>
</html>"#;
    let results = linter.lint(html).unwrap();

    assert!(results.len() >= 4, "Should detect multiple violations");
    assert!(
        results.iter().any(|r| r.rule == "no-duplicate-attrs"),
        "Should detect duplicate class attributes"
    );
    assert!(
        results.iter().any(|r| r.rule == "no-duplicate-id"),
        "Should detect duplicate IDs"
    );
    assert!(
        results.iter().any(|r| r.rule == "no-inline-styles"),
        "Should detect inline styles"
    );
    assert!(
        results.iter().any(|r| r.rule == "require-lang"),
        "Should detect missing lang attribute"
    );
}

#[test]
fn test_valid_attributes() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let html = r#"<!DOCTYPE html>
<html lang="en">
    <div class="foo bar" id="unique">
        <span class="baz">Valid attributes</span>
    </div>
</html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        !results.iter().any(|r| r.rule == "no-duplicate-attrs"),
        "Should not flag valid multiple classes"
    );
    assert!(
        !results.iter().any(|r| r.rule == "no-duplicate-id"),
        "Should not flag valid unique IDs"
    );
}

#[test]
fn test_doctype_variations() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let test_cases = vec![
        (
            r#"<!DOCTYPE html><html lang="en"></html>"#,
            true,
            "HTML5 DOCTYPE",
        ),
        (
            r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"><html lang="en"></html>"#,
            true,
            "HTML 4.01 DOCTYPE",
        ),
        (r#"<html lang="en"></html>"#, false, "Missing DOCTYPE"),
    ];

    for (html, should_pass, message) in test_cases {
        let results = linter.lint(html).unwrap();
        let has_doctype_error = results.iter().any(|r| r.rule == "require-doctype");
        assert_eq!(!has_doctype_error, should_pass, "{}", message);
    }
}

#[test]
fn test_lang_attribute_values() {
    let linter = HtmlLinter::new(setup_eslint_rules(), None);
    let test_cases = vec![
        (
            r#"<!DOCTYPE html><html lang="en"></html>"#,
            true,
            "Simple language code",
        ),
        (
            r#"<!DOCTYPE html><html lang="en-US"></html>"#,
            true,
            "Language code with region",
        ),
        (
            r#"<!DOCTYPE html><html lang=""></html>"#,
            false,
            "Empty lang attribute",
        ),
        (
            r#"<!DOCTYPE html><html></html>"#,
            false,
            "Missing lang attribute",
        ),
    ];

    for (html, should_pass, message) in test_cases {
        let results = linter.lint(html).unwrap();
        let has_lang_error = results.iter().any(|r| r.rule == "require-lang");
        assert_eq!(!has_lang_error, should_pass, "{}", message);
    }
}
