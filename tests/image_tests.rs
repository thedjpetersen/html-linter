use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use std::collections::HashMap;

fn setup_image_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "img-dimensions".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "img".to_string(),
            condition: "dimensions-present".to_string(),
            message: "Images should not specify width and height attributes - use CSS instead"
                .to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("check_mode".to_string(), "ensure_nonexistence".to_string());
                options.insert("attributes".to_string(), "width,height".to_string());
                options.insert("pattern".to_string(), r#"^\d+$"#.to_string());
                options
            },
        },
        Rule {
            name: "img-loading".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "img".to_string(),
            condition: "loading-attribute".to_string(),
            message: "Images should have a loading attribute with value 'lazy' or 'eager'"
                .to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("pattern".to_string(), r#"^(lazy|eager)$"#.to_string());
                options.insert("attributes".to_string(), "loading".to_string());
                options
            },
        },
        Rule {
            name: "img-format".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "img".to_string(),
            condition: "file-extension".to_string(),
            message: "Use modern image formats".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "pattern".to_string(),
                    r#".*\.(webp|avif|jpg|jpeg|png)$"#.to_string(),
                );
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("attributes".to_string(), "src".to_string());
                options
            },
        },
        Rule {
            name: "img-filename".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "img".to_string(),
            condition: "filename-pattern".to_string(),
            message: "Image filenames should be descriptive".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), r#"^[a-z0-9-]+\.[a-z]+$"#.to_string());
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("attributes".to_string(), "src".to_string());
                options
            },
        },
    ]
}

#[test]
fn test_image_with_all_best_practices() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img 
        src="hero-banner.webp" 
        alt="Company hero banner" 
        loading="lazy"
    >"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(results.len(), 0, "Expected no violations for good image");
}

#[test]
fn test_image_without_dimensions_passes() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="test.jpg" alt="Test image" loading="lazy">"#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Should not flag images without dimensions"
    );
}

#[test]
fn test_image_with_both_dimensions_warns() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="test.jpg" alt="Test image" width="100" height="100" loading="lazy">"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "img-dimensions"),
        "Should warn when both width and height are present"
    );
}

#[test]
fn test_image_with_only_width_warns() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="test.jpg" alt="Test image" width="100" loading="lazy">"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "img-dimensions"),
        "Should warn when width is present"
    );
}

#[test]
fn test_image_with_only_height_warns() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="test.jpg" alt="Test image" height="100" loading="lazy">"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "img-dimensions"),
        "Should warn when height is present"
    );
}

#[test]
fn test_image_missing_loading_attribute() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="image.webp" alt="Test" width="800" height="400">"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "img-loading"),
        "Should detect missing loading attribute"
    );
}

#[test]
fn test_image_invalid_format() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="image.gif" alt="Test" width="800" height="400" loading="lazy">"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "img-format"),
        "Should detect invalid image format"
    );
}

#[test]
fn test_image_non_descriptive_filename() {
    let linter = HtmlLinter::new(setup_image_rules(), None);
    let html = r#"<img src="IMG_12345.jpg" alt="Test" width="800" height="400" loading="lazy">"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "img-filename"),
        "Should detect non-descriptive filename"
    );
}
