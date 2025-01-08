use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use serde_json::json;
use std::collections::HashMap;

fn setup_button_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "button-type".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "button".to_string(),
            condition: "explicit-type".to_string(),
            message: "Buttons should have an explicit type attribute".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "pattern".to_string(),
                    r#"^(submit|button|reset)$"#.to_string(),
                );
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("attributes".to_string(), "type".to_string());
                options
            },
        },
        Rule {
            name: "button-accessible-name".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Error,
            selector: "button".to_string(),
            condition: "any-condition-met".to_string(),
            message: "Buttons must have an accessible name via text content, aria-label, or aria-labelledby".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("check_mode".to_string(), "any".to_string());
                options.insert("conditions".to_string(), json!([
                    {
                        "type": "TextContent",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "aria-label",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "aria-labelledby",
                        "pattern": r#".+"#,
                    }
                ]).to_string());
                options
            },
        },
        Rule {
            name: "button-no-disabled".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "button[disabled]".to_string(),
            condition: "aria-disabled".to_string(),
            message: "Consider using aria-disabled instead of disabled attribute".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options.insert("attributes".to_string(), "disabled".to_string());
                options.insert("pattern".to_string(), r#".*"#.to_string());
                options
            },
        },
    ]
}

#[test]
fn test_button_with_all_best_practices() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="submit" aria-disabled="false">Submit Form</button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0, "Expected no violations for good button");
}

#[test]
fn test_button_missing_type() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button>Submit</button>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "button-type"),
        "Should warn about missing type attribute"
    );
}

#[test]
fn test_button_empty_text() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="button"></button>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "button-accessible-name"),
        "Should error about missing accessible name"
    );
}

#[test]
fn test_button_with_disabled_attribute() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="submit" disabled>Submit</button>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "button-no-disabled"),
        "Should warn about using disabled attribute instead of aria-disabled"
    );
}

#[test]
fn test_button_with_aria_label() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="button" aria-label="Close dialog"></button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Should accept buttons with aria-label even without text content"
    );
}

#[test]
fn test_button_compound_rule_cases() {
    let linter = HtmlLinter::new(setup_button_rules(), None);

    // Test cases showing different ways to satisfy the compound rule
    let test_cases = vec![
        (
            r#"<button type="button">Submit Form</button>"#,
            true,
            "Text content should satisfy the rule",
        ),
        (
            r#"<button type="button" aria-label="Close dialog"></button>"#,
            true,
            "aria-label should satisfy the rule",
        ),
        (
            r#"
                <span id="btn-label">Save changes</span>
                <button type="button" aria-labelledby="btn-label"></button>
            "#,
            true,
            "aria-labelledby should satisfy the rule",
        ),
        (
            r#"<button type="button">  </button>"#,
            false,
            "Whitespace-only content should not satisfy the rule",
        ),
        (
            r#"<button type="button" aria-label=""></button>"#,
            false,
            "Empty aria-label should not satisfy the rule",
        ),
    ];

    for (html, should_pass, message) in test_cases {
        let results = linter.lint(html).unwrap();
        let has_violation = results.iter().any(|r| r.rule == "button-accessible-name");
        assert_eq!(!has_violation, should_pass, "{}", message);
    }
}

#[test]
fn test_button_with_text_content() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="button">Submit Form</button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button with text content should pass accessibility check"
    );
}

#[test]
fn test_button_with_aria_labelledby() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"
        <span id="btn-label">Save changes</span>
        <button type="button" aria-labelledby="btn-label"></button>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button with valid aria-labelledby should pass accessibility check"
    );
}

#[test]
fn test_button_with_whitespace_content() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="button">  </button>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "button-accessible-name"),
        "Button with only whitespace content should fail accessibility check"
    );
}

#[test]
fn test_button_with_empty_aria_label() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="button" aria-label=""></button>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "button-accessible-name"),
        "Button with empty aria-label should fail accessibility check"
    );
}

#[test]
fn test_button_with_nonexistent_labelledby() {
    // Define a specific rule for this test
    let rule = Rule {
        name: "button-accessible-name".to_string(),
        rule_type: RuleType::Compound,
        severity: Severity::Error,
        selector: "button".to_string(),
        condition: "any-condition-met".to_string(),
        message:
            "Buttons must have an accessible name via text content, aria-label, or aria-labelledby"
                .to_string(),
        options: {
            let mut options = HashMap::new();
            options.insert("check_mode".to_string(), "any".to_string());
            options.insert(
                "conditions".to_string(),
                json!([
                    {
                        "type": "TextContent",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "aria-label",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeReference",
                        "attribute": "aria-labelledby",
                        "reference_must_exist": true,
                        "reference_type": "id"
                    }
                ])
                .to_string(),
            );
            options
        },
    };

    let linter = HtmlLinter::new(vec![rule], None);
    let html = r#"<button type="button" aria-labelledby="nonexistent"></button>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "button-accessible-name"),
        "Button with non-existent aria-labelledby reference should fail accessibility check"
    );
}

#[test]
fn test_button_with_multiple_conditions() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"
        <span id="btn-label">Save</span>
        <button 
            type="button" 
            aria-label="Save changes" 
            aria-labelledby="btn-label"
        >
            Save document
        </button>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button satisfying multiple accessibility conditions should pass"
    );
}

#[test]
fn test_button_with_no_accessible_name() {
    let linter = HtmlLinter::new(setup_button_rules(), None);
    let html = r#"<button type="button"></button>"#;
    let results = linter.lint(html).unwrap();

    let violations: Vec<_> = results
        .iter()
        .filter(|r| r.rule == "button-accessible-name")
        .collect();

    assert_eq!(
        violations.len(),
        1,
        "Should have exactly one accessibility violation"
    );

    let violation = &violations[0];
    assert_eq!(
        violation.severity,
        Severity::Error,
        "Accessibility violation should be an error"
    );
    assert!(
        violation.message.contains("accessible name"),
        "Error message should mention accessible name requirement"
    );
}

#[test]
fn test_button_weighted_conditions() {
    let mut rules = setup_button_rules();
    rules.push(Rule {
        name: "button-weighted-accessibility".to_string(),
        rule_type: RuleType::Compound,
        severity: Severity::Warning,
        selector: "button".to_string(),
        condition: "weighted-conditions".to_string(),
        message: "Button should meet weighted accessibility requirements".to_string(),
        options: {
            let mut options = HashMap::new();
            options.insert("check_mode".to_string(), "weighted".to_string());
            options.insert("weights".to_string(), json!([2.0, 1.0, 0.5]).to_string());
            options.insert("threshold".to_string(), "2.5".to_string());
            options.insert(
                "conditions".to_string(),
                json!([
                    {
                        "type": "TextContent",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "aria-label",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "role",
                        "pattern": r#"^button$"#,
                    }
                ])
                .to_string(),
            );
            options
        },
    });

    let linter = HtmlLinter::new(rules, None);

    // Test passing case - text content (2.0) + aria-label (1.0) > threshold (2.5)
    let html = r#"<button type="button" aria-label="Save">Save Changes</button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button meeting weight threshold should pass"
    );

    // Test failing case - only text content (2.0) < threshold (2.5)
    let html = r#"<button type="button">Save Changes</button>"#;
    let results = linter.lint(html).unwrap();
    assert!(results
        .iter()
        .any(|r| r.rule == "button-weighted-accessibility"));
}

#[test]
fn test_button_dependency_chain() {
    let mut rules = vec![];
    rules.push(Rule {
        name: "button-progressive-enhancement".to_string(),
        rule_type: RuleType::Compound,
        severity: Severity::Warning,
        selector: "button".to_string(),
        condition: "dependency-chain".to_string(),
        message: "Button should follow progressive enhancement pattern".to_string(),
        options: {
            let mut options = HashMap::new();
            options.insert("check_mode".to_string(), "dependency_chain".to_string());
            options.insert(
                "conditions".to_string(),
                json!([
                    {
                        "type": "TextContent",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "aria-label",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "data-enhanced",
                        "pattern": r#"true"#,
                    }
                ])
                .to_string(),
            );
            options
        },
    });

    let linter = HtmlLinter::new(rules, None);

    // Test valid dependency chain
    let html =
        r#"<button type="button" aria-label="Save" data-enhanced="true">Save Changes</button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button with valid dependency chain should pass"
    );

    // Test broken dependency chain - missing text content and aria-label but has data-enhanced,
    // which violates the required dependency order of: text -> aria-label -> data-enhanced
    let html = r#"<button type="button" data-enhanced="true"></button>"#;
    let results = linter.lint(html).unwrap();
    assert!(results
        .iter()
        .any(|r| r.rule == "button-progressive-enhancement"));
}

#[test]
fn test_button_alternating_pattern() {
    let mut rules = vec![];
    rules.push(Rule {
        name: "button-icon-text-pattern".to_string(),
        rule_type: RuleType::Compound,
        severity: Severity::Warning,
        selector: "button".to_string(),
        condition: "alternating-pattern".to_string(),
        message: "Button should alternate between icon and text".to_string(),
        options: {
            let mut options = HashMap::new();
            options.insert("check_mode".to_string(), "alternating".to_string());
            options.insert(
                "conditions".to_string(),
                json!([
                    {
                        "type": "AttributeValue",
                        "attribute": "class",
                        "pattern": r#"icon-"#,
                    },
                    {
                        "type": "TextContent",
                        "pattern": r#".+"#,
                    }
                ])
                .to_string(),
            );
            options
        },
    });

    let linter = HtmlLinter::new(rules, None);

    // Test valid alternating pattern - expect to fail
    let html = r#"<button type="button" class="icon-save">Save Changes</button>"#;
    let results = linter.lint(html).unwrap();
    assert!(results.iter().any(|r| r.rule == "button-icon-text-pattern"));

    // Test invalid pattern (two text elements)
    let html = r#"<button type="button">Save Save</button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button with alternating icon-text pattern should pass"
    );
}

#[test]
fn test_button_subset_match() {
    let mut rules = setup_button_rules();
    rules.push(Rule {
        name: "button-valid-combinations".to_string(),
        rule_type: RuleType::Compound,
        severity: Severity::Warning,
        selector: "button".to_string(),
        condition: "valid-combinations".to_string(),
        message: "Button should use valid combination of attributes".to_string(),
        options: {
            let mut options = HashMap::new();
            options.insert("check_mode".to_string(), "subset_match".to_string());
            options.insert(
                "valid_sets".to_string(),
                json!([
                    [0, 1],    // text + aria-label
                    [0, 2],    // text + role
                    [1, 2, 3]  // aria-label + role + data-test
                ])
                .to_string(),
            );
            options.insert(
                "conditions".to_string(),
                json!([
                    {
                        "type": "TextContent",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "aria-label",
                        "pattern": r#".+"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "role",
                        "pattern": r#"button"#,
                    },
                    {
                        "type": "AttributeValue",
                        "attribute": "data-test",
                        "pattern": r#".+"#,
                    }
                ])
                .to_string(),
            );
            options
        },
    });

    let linter = HtmlLinter::new(rules, None);

    // Test valid combination (text + aria-label)
    let html = r#"<button type="button" aria-label="Save">Save Changes</button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button with valid attribute combination should pass"
    );

    // Test invalid combination (only text)
    let html = r#"<button type="button">Save Changes</button>"#;
    let results = linter.lint(html).unwrap();
    assert!(results
        .iter()
        .any(|r| r.rule == "button-valid-combinations"));

    // Test valid combination (aria-label + role + data-test)
    let html =
        r#"<button type="button" aria-label="Save" role="button" data-test="save-btn"></button>"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Button with valid complex combination should pass"
    );
}
