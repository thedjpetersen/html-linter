use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use serde_json::json;
use std::collections::HashMap;

fn create_basic_linter() -> HtmlLinter {
    let rules = vec![
        Rule {
            name: "img-alt".to_string(),
            rule_type: RuleType::AttributePresence,
            severity: Severity::Error,
            selector: "img".to_string(),
            condition: "alt-missing".to_string(),
            message: "Images must have alt attributes".to_string(),
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
    ];

    HtmlLinter::new(rules, None)
}

#[test]
fn test_img_alt_attribute() {
    let linter = create_basic_linter();

    // Test missing alt attribute
    let html = r#"<html><body><img src="test.jpg"></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule, "img-alt");
    assert_eq!(results[0].severity, Severity::Error);

    // Test with alt attribute
    let html = r#"<html><body><img src="test.jpg" alt="Test image"></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_inline_styles() {
    let linter = create_basic_linter();

    // Test with inline style
    let html = r#"<div style="color: red;">Test</div>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule, "no-inline-styles");
    assert_eq!(results[0].severity, Severity::Warning);

    // Test without inline style
    let html = r#"<div class="red">Test</div>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_heading_order() {
    let rules = vec![Rule {
        name: "heading-order".to_string(),
        rule_type: RuleType::ElementOrder,
        severity: Severity::Error,
        selector: "h1,h2,h3,h4,h5,h6".to_string(),
        condition: "sequential-order".to_string(),
        message: "Heading levels should not be skipped".to_string(),
        options: HashMap::new(),
    }];

    let linter = HtmlLinter::new(rules, None);

    // Test correct heading order
    let html = r#"<h1>Title</h1><h2>Subtitle</h2><h3>Section</h3>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);

    // Test incorrect heading order
    let html = r#"<h1>Title</h1><h3>Skipped h2</h3>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_semantic_structure() {
    let rules = vec![Rule {
        name: "semantic-structure".to_string(),
        rule_type: RuleType::AttributeValue,
        severity: Severity::Warning,
        selector: "div,span".to_string(),
        condition: "attribute-value".to_string(),
        message: "Consider using semantic HTML elements".to_string(),
        options: {
            let mut options = HashMap::new();
            options.insert("attributes".to_string(), "class".to_string());
            options.insert(
                "pattern".to_string(),
                "^(header|main|footer|article|section|nav|navigation)$".to_string(),
            );
            options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
            options
        },
    }];

    let linter = HtmlLinter::new(rules, None);

    // Test cases that should trigger warnings
    let test_cases = vec![
        (
            r#"<div class="header">Header content</div>"#,
            true,
            "div with header class",
        ),
        (
            r#"<div class="footer">Footer content</div>"#,
            true,
            "div with footer class",
        ),
        (
            r#"<div class="navigation">Nav content</div>"#,
            true,
            "div with navigation class",
        ),
        (
            r#"<div class="main">Main content</div>"#,
            true,
            "div with main class",
        ),
    ];

    // Test cases that should not trigger warnings
    let negative_test_cases = vec![
        (
            r#"<header>Header content</header>"#,
            false,
            "semantic header element",
        ),
        (
            r#"<div class="custom">Custom content</div>"#,
            false,
            "div without semantic class",
        ),
        (
            r#"<main>Main content</main>"#,
            false,
            "semantic main element",
        ),
    ];

    // Run positive test cases
    for (html, should_warn, test_name) in test_cases {
        let results = linter.lint(html).unwrap();
        assert_eq!(
            results.len(),
            if should_warn { 1 } else { 0 },
            "Failed test case: {}",
            test_name
        );
        if should_warn {
            assert_eq!(results[0].severity, Severity::Warning);
            assert_eq!(results[0].rule, "semantic-structure");
        }
    }

    // Run negative test cases
    for (html, should_warn, test_name) in negative_test_cases {
        let results = linter.lint(html).unwrap();
        assert_eq!(
            results.len(),
            if should_warn { 1 } else { 0 },
            "Failed test case: {}",
            test_name
        );
    }
}

#[test]
fn test_nested_elements() {
    let rules = vec![Rule {
        name: "input-label".to_string(),
        rule_type: RuleType::Nesting,
        severity: Severity::Error,
        selector: "input".to_string(),
        condition: "parent-label-or-for".to_string(),
        message: "Input elements should be associated with a label".to_string(),
        options: HashMap::new(),
    }];

    let linter = HtmlLinter::new(rules, None);

    // Test input without label
    let html = r#"<div><input type="text"></div>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 1);

    // Test input with label
    let html = r#"<label>Name: <input type="text"></label>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);

    // Test input with label using 'for' attribute
    let html = r#"<label for="name">Name:</label><input id="name" type="text">"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_seo_rules() {
    let rules = vec![
        Rule {
            name: "meta-description".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Error,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Meta description validation failed".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    json!([{
                        "name": "description",
                        "pattern": {
                            "type": "MinLength",
                            "value": 50
                        },
                        "required": true
                    }])
                    .to_string(),
                );
                options
            },
        },
        Rule {
            name: "og-tags".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Warning,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Open Graph tag validation failed".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    json!([
                        {
                            "property": "og:type",
                            "pattern": {
                                "type": "OneOf",
                                "value": ["website", "article", "product"]
                            },
                            "required": true
                        },
                        {
                            "property": "og:title",
                            "pattern": {
                                "type": "NonEmpty"
                            },
                            "required": true
                        }
                    ])
                    .to_string(),
                );
                options
            },
        },
        Rule {
            name: "viewport".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Error,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Viewport meta tag validation failed".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    json!([{
                        "name": "viewport",
                        "pattern": {
                            "type": "Exact",
                            "value": "width=device-width, initial-scale=1"
                        },
                        "required": true
                    }])
                    .to_string(),
                );
                options
            },
        },
    ];

    let linter = HtmlLinter::new(rules, None);

    // Test missing meta description
    let html = r#"<html><head><title>Page Title</title></head><body></body></html>"#;
    let unwrapped_results = linter.lint(html);
    let results: Vec<html_linter::LintResult> = unwrapped_results.unwrap();
    assert!(results.iter().any(|r| r.rule == "meta-description"));

    // Test meta description too short
    let html = r#"<html><head>
        <meta name="description" content="Too short">
    </head><body></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(results.iter().any(|r| r.rule == "meta-description"));

    // Test missing Open Graph tags
    let html = r#"<html><head>
        <meta name="description" content="This is a properly lengthy description that should meet the minimum length requirement for SEO purposes.">
    </head><body></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(results.iter().any(|r| r.rule == "og-tags"));

    // Test invalid viewport
    let html = r#"<html><head>
        <meta name="description" content="This is a properly lengthy description that should meet the minimum length requirement for SEO purposes.">
        <meta property="og:type" content="website">
        <meta property="og:title" content="Page Title">
        <meta name="viewport" content="width=device-width">
    </head><body></body></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(results.iter().any(|r| r.rule == "viewport"));

    // Test valid meta tags
    let html = r#"
        <html>
            <head>
                <meta name="description" content="This is a properly lengthy description that should meet the minimum length requirement for SEO purposes.">
                <meta property="og:type" content="website">
                <meta property="og:title" content="Page Title">
                <meta name="viewport" content="width=device-width, initial-scale=1">
            </head>
            <body>
                <h1>Main Page Heading</h1>
                <p>Content</p>
            </body>
        </html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0, "Expected no validation errors");
}

#[test]
fn test_load_rules_from_json() {
    // Test valid JSON
    let json = r#"[
        {
            "name": "test-rule",
            "rule_type": "ElementPresence",
            "severity": "Error",
            "selector": "div",
            "condition": "required",
            "message": "Test message",
            "options": {}
        }
    ]"#;

    let linter = HtmlLinter::from_json(json, None).unwrap();
    let rules = linter.get_rules();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "test-rule");
    assert_eq!(rules[0].message, "Test message");

    // Test invalid JSON
    let invalid_json = r#"[
        {
            "name": "test-rule",
            "invalid_field": "value"
        }
    ]"#;

    let result = HtmlLinter::from_json(invalid_json, None);
    assert!(result.is_err());
}

#[test]
fn test_load_rules_from_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create temporary file with valid rules
    let mut temp_file = NamedTempFile::new().unwrap();
    let json_content = r#"[
        {
            "name": "file-rule",
            "rule_type": "ElementPresence",
            "severity": "Warning",
            "selector": "span",
            "condition": "required",
            "message": "File test message",
            "options": {}
        }
    ]"#;
    write!(temp_file, "{}", json_content).unwrap();

    // Test loading valid file
    let linter = HtmlLinter::from_json_file(temp_file.path().to_str().unwrap(), None).unwrap();
    let rules = linter.get_rules();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "file-rule");
    assert_eq!(rules[0].severity, Severity::Warning);

    // Test loading non-existent file
    let result = HtmlLinter::from_json_file("non_existent_file.json", None);
    assert!(result.is_err());
}

#[test]
fn test_load_complex_rules() {
    let json = r#"[
        {
            "name": "meta-tags",
            "rule_type": "ElementContent",
            "severity": "Error",
            "selector": "head",
            "condition": "meta-tags",
            "message": "Meta tags validation failed",
            "options": {
                "required_meta_tags": "[{\"name\":\"description\",\"pattern\":{\"type\":\"MinLength\",\"value\":50},\"required\":true}]"
            }
        },
        {
            "name": "semantic-elements",
            "rule_type": "Semantics",
            "severity": "Warning",
            "selector": "div,span",
            "condition": "semantic-structure",
            "message": "Use semantic elements where appropriate",
            "options": {
                "semantic_alternatives": "[\"header\",\"main\",\"footer\",\"article\",\"section\",\"nav\"]"
            }
        }
    ]"#;

    let linter = HtmlLinter::from_json(json, None).unwrap();
    let rules = linter.get_rules();
    assert_eq!(rules.len(), 2);

    // Test first rule
    assert_eq!(rules[0].name, "meta-tags");
    assert!(matches!(rules[0].rule_type, RuleType::ElementContent));
    assert!(rules[0].options.contains_key("required_meta_tags"));

    // Test second rule
    assert_eq!(rules[1].name, "semantic-elements");
    assert!(matches!(rules[1].rule_type, RuleType::Semantics));
    assert!(rules[1].options.contains_key("semantic_alternatives"));
}
