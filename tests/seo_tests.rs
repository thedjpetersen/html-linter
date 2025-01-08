use html_linter::{HtmlLinter, Rule, RuleType, Severity};
use std::collections::HashMap;

fn setup_seo_rules() -> Vec<Rule> {
    vec![
        // Core Meta Tags
        Rule {
            name: "meta-description".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Error,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Meta description must be between 50 and 160 characters".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    r#"[{
                        "name": "description",
                        "pattern": {
                            "type": "LengthRange",
                            "min": 50,
                            "max": 160
                        },
                        "required": true
                    }]"#
                    .to_string(),
                );
                options
            },
        },
        Rule {
            name: "meta-title".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Error,
            selector: "head title".to_string(),
            condition: "content-length".to_string(),
            message: "Title tag must be between 30 and 60 characters".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("min_length".to_string(), "30".to_string());
                options.insert("max_length".to_string(), "60".to_string());
                options
            },
        },
        // Add this rule after the meta-title rule and before the meta-robots-advanced rule
        Rule {
            name: "canonical-url".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Error,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Canonical URL must be present and valid".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    r#"[{
                        "rel": "canonical",
                        "pattern": {
                            "type": "Regex",
                            "value": "^https?://[\\w.-]+\\.[a-zA-Z]{2,}(?:/[\\w.-]*)*/?$"
                        },
                        "required": true
                    }]"#
                    .to_string(),
                );
                options
            },
        },
        // Advanced Meta Tags
        Rule {
            name: "meta-robots-advanced".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Warning,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Advanced robots meta directives should be properly configured".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    r#"[{
                        "name": "robots",
                        "pattern": {
                            "type": "OneOf",
                            "value": [
                                "index, follow",
                                "index, follow, max-snippet:-1, max-image-preview:large",
                                "noindex, follow",
                                "index, nofollow",
                                "noindex, nofollow"
                            ]
                        },
                        "required": true
                    }]"#
                    .to_string(),
                );
                options
            },
        },
        // Social Media Optimization
        Rule {
            name: "og-tags-complete".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Warning,
            selector: "head".to_string(),
            condition: "meta-tags".to_string(),
            message: "Complete Open Graph tags required for optimal social sharing".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_meta_tags".to_string(),
                    r#"[
                        {
                            "property": "og:title",
                            "pattern": {
                                "type": "LengthRange",
                                "min": 30,
                                "max": 60
                            },
                            "required": true
                        },
                        {
                            "property": "og:description",
                            "pattern": {
                                "type": "LengthRange",
                                "min": 50,
                                "max": 200
                            },
                            "required": true
                        },
                        {
                            "property": "og:image",
                            "pattern": {
                                "type": "Regex",
                                "value": "^https://.+\\.(jpg|jpeg|png|webp)$"
                            },
                            "required": true
                        },
                        {
                            "property": "og:url",
                            "pattern": {
                                "type": "Regex",
                                "value": "^https://"
                            },
                            "required": true
                        }
                    ]"#
                    .to_string(),
                );
                options
            },
        },
        // Performance and Core Web Vitals
        Rule {
            name: "resource-loading".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "script:not([type='application/ld+json']), link[rel='stylesheet']"
                .to_string(),
            condition: "loading-optimization".to_string(),
            message: "Resource loading should be optimized for Core Web Vitals".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("attributes".to_string(), "defer,async,loading".to_string());
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("pattern".to_string(), r#"^(lazy|eager|auto|\d+)$"#.to_string());
                options
            },
        },
        // Structured Data
        Rule {
            name: "structured-data-required".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Error,
            selector: "head script[type='application/ld+json']".to_string(),
            condition: "json-ld-validation".to_string(),
            message: "Required structured data missing or invalid".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "required_schemas".to_string(),
                    r#"["WebPage", "Organization", "BreadcrumbList"]"#.to_string(),
                );
                options
            },
        },
        // Content Optimization
        Rule {
            name: "heading-optimization".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "h1,h2,h3".to_string(),
            condition: "content-optimization".to_string(),
            message: "Heading structure should be optimized for SEO".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "TextContent",
                            "pattern": "^.{10,60}$"
                        },
                        {
                            "type": "AttributeValue",
                            "attribute": "id",
                            "pattern": "^[a-z0-9-]+$"
                        }
                    ]"#
                    .to_string(),
                );
                options.insert("check_mode".to_string(), "all".to_string());
                options
            },
        },
        // International SEO
        Rule {
            name: "hreflang-implementation".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "link[rel='alternate'][hreflang]".to_string(),
            condition: "valid-hreflang".to_string(),
            message: "Complete hreflang implementation required for international SEO".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("pattern".to_string(), r#"^[a-z]{2}(-[A-Z]{2})?$"#.to_string());
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert("attributes".to_string(), "hreflang".to_string());
                options
            },
        },
        // Mobile Optimization
        Rule {
            name: "mobile-optimization".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Error,
            selector: "head".to_string(),
            condition: "mobile-friendly".to_string(),
            message: "Page must be optimized for mobile devices".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "meta[name='viewport']",
                            "attribute": "content",
                            "pattern": "width=device-width, initial-scale=1",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options.insert("check_mode".to_string(), "all".to_string());
                options
            },
        },
        // Image Optimization
        Rule {
            name: "image-optimization".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "img".to_string(),
            condition: "image-best-practices".to_string(),
            message: "Images must follow SEO best practices".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "attributes".to_string(),
                    "alt,loading,width,height".to_string(),
                );
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options.insert(
                    "pattern".to_string(),
                    r#"^(lazy|eager|auto|\d+)$"#.to_string(),
                );
                options
            },
        },
        // URL Structure
        Rule {
            name: "url-structure".to_string(),
            rule_type: RuleType::AttributeValue,
            severity: Severity::Warning,
            selector: "a[href]".to_string(),
            condition: "url-best-practices".to_string(),
            message: "URLs should follow SEO best practices".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "pattern".to_string(),
                    r#"^(/[a-z0-9-]+)*/?$|^https?://"#.to_string(),
                );
                options.insert("check_mode".to_string(), "ensure_existence".to_string());
                options
            },
        },
        // Core Web Vitals Optimization
        Rule {
            name: "core-web-vitals".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "body".to_string(),
            condition: "performance-optimization".to_string(),
            message: "Page should be optimized for Core Web Vitals".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "img",
                            "attribute": "loading",
                            "pattern": "^lazy$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "link",
                            "attribute": "rel",
                            "pattern": "^(preload|prefetch|preconnect)$",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options.insert("check_mode".to_string(), "any".to_string());
                options
            },
        },
        // AI-Optimized Content Structure
        Rule {
            name: "ai-readiness".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "article, main, .content".to_string(),
            condition: "content-structure".to_string(),
            message: "Content structure should be optimized for AI crawlers and LLMs".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "h1 + p, h2 + p, h3 + p",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "TextContent",
                            "pattern": "^(?!.*(?:click here|read more|learn more)$).+$"
                        },
                        {
                            "type": "AttributeValue",
                            "attribute": "data-content-type",
                            "pattern": "^(article|guide|tutorial|product)$",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options.insert("check_mode".to_string(), "all".to_string());
                options
            },
        },
        // E-E-A-T Signals
        Rule {
            name: "eat-signals".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "body".to_string(),
            condition: "expertise-signals".to_string(),
            message: "Page should demonstrate Experience, Expertise, Authoritativeness, and Trustworthiness".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "article[itemtype*='Article'] [itemprop='author']",
                            "attribute": "itemprop",
                            "pattern": "^author$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "time",
                            "attribute": "itemprop",
                            "pattern": "^(datePublished|dateModified)$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": ".credentials, .author-bio, [itemprop='citation']",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // User Experience Signals
        Rule {
            name: "ux-signals".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Error,
            selector: "body".to_string(),
            condition: "user-experience".to_string(),
            message: "Page must meet Core Web Vitals and UX requirements".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "main[role='main'], [aria-label]",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "*",
                            "attribute": "class",
                            "pattern": ".*(?:container|wrapper|content).*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "div > div > div > div > div",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_nonexistence"
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // Content Hierarchy and Semantic Structure
        Rule {
            name: "semantic-structure".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "body".to_string(),
            condition: "semantic-html".to_string(),
            message: "Content must use semantic HTML elements appropriately".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "header, main, footer, nav, article, section, aside",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "div > div > div > div > div",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_nonexistence"
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // Advanced Schema Implementation
        Rule {
            name: "schema-hierarchy".to_string(),
            rule_type: RuleType::ElementContent,
            severity: Severity::Warning,
            selector: "script[type='application/ld+json']".to_string(),
            condition: "schema-validation".to_string(),
            message: "Schema markup should implement proper hierarchy and relationships".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "schema_requirements".to_string(),
                    r#"{
                        "WebPage": {
                            "required": ["breadcrumb", "mainEntity"],
                            "recommended": ["speakable", "reviewedBy"]
                        },
                        "Article": {
                            "required": ["author", "datePublished", "dateModified"],
                            "recommended": ["citation", "backstory", "speakable"]
                        },
                        "Product": {
                            "required": ["offers", "aggregateRating"],
                            "recommended": ["review", "brand", "manufacturer"]
                        }
                    }"#.to_string(),
                );
                options
            },
        },
        // Content Readability and Engagement
        Rule {
            name: "content-quality".to_string(),
            rule_type: RuleType::TextContent,
            severity: Severity::Warning,
            selector: "article p, article li".to_string(),
            condition: "readability-check".to_string(),
            message: "Content should meet readability and engagement standards".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "patterns".to_string(),
                    r#"[
                        {
                            "type": "SentenceLength",
                            "max": 25
                        },
                        {
                            "type": "ParagraphLength",
                            "max": 150
                        },
                        {
                            "type": "ReadingLevel",
                            "max": 8
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // Technical Performance Optimization
        Rule {
            name: "performance-optimization".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Error,
            selector: "html".to_string(),
            condition: "performance-check".to_string(),
            message: "Page must implement advanced performance optimizations".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "link[rel='preconnect'], link[rel='preload'], link[rel='prefetch']",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "Compound",
                            "selector": "img, picture",
                            "conditions": [
                                {
                                    "type": "AttributeValue",
                                    "selector": "source[type='image/webp']",
                                    "attribute": "srcset",
                                    "pattern": ".*\\.webp(\\s+\\d+[wx])?(,\\s*.*\\.webp(\\s+\\d+[wx])?)*",
                                    "check_mode": "ensure_existence"
                                },
                                {
                                    "type": "AttributeValue",
                                    "selector": "source[type='image/avif']",
                                    "attribute": "srcset",
                                    "pattern": ".*\\.avif(\\s+\\d+[wx])?(,\\s*.*\\.avif(\\s+\\d+[wx])?)*",
                                    "check_mode": "ensure_existence"
                                },
                                {
                                    "type": "AttributeValue",
                                    "selector": "img",
                                    "attribute": "loading",
                                    "pattern": "^lazy$",
                                    "check_mode": "ensure_existence"
                                },
                                {
                                    "type": "AttributeValue",
                                    "selector": "img",
                                    "attribute": "decoding",
                                    "pattern": "^async$",
                                    "check_mode": "ensure_existence"
                                },
                                {
                                    "type": "AttributeValue",
                                    "selector": "img, source",
                                    "attribute": "sizes",
                                    "pattern": "^\\([^)]+\\)\\s+\\d+[vw]px(,\\s*\\([^)]+\\)\\s+\\d+[vw]px)*",
                                    "check_mode": "ensure_existence"
                                }
                            ],
                            "check_mode": "all"
                        },
                        {
                            "type": "Compound",
                            "selector": "script:not([type='application/ld+json'])",
                            "conditions": [
                                {
                                    "type": "AttributeValue",
                                    "attribute": "type",
                                    "pattern": "^(module|application/javascript)$",
                                    "check_mode": "ensure_existence"
                                },
                                {
                                    "type": "AttributeValue",
                                    "attribute": "async|defer",
                                    "pattern": "^(|async|defer)$",
                                    "check_mode": "ensure_existence"
                                },
                                {
                                    "type": "AttributeValue",
                                    "attribute": "src",
                                    "pattern": "^(https://|/)",
                                    "check_mode": "ensure_existence"
                                }
                            ],
                            "check_mode": "all"
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // Progressive Enhancement
        Rule {
            name: "progressive-enhancement".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "body".to_string(),
            condition: "enhancement-check".to_string(),
            message: "Implement progressive enhancement for better accessibility and performance".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "noscript",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "picture > img, video > p",
                            "attribute": "*",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "style[data-critical]",
                            "attribute": "data-critical",
                            "pattern": ".*",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // International and Language Optimization
        Rule {
            name: "language-optimization".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "html".to_string(),
            condition: "language-check".to_string(),
            message: "Implement proper language and international optimization".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "html",
                            "attribute": "lang",
                            "pattern": "^[a-z]{2}(-[A-Z]{2})?$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "[lang]",
                            "attribute": "lang",
                            "pattern": "^[a-z]{2}(-[A-Z]{2})?$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "[data-i18n]",
                            "attribute": "data-i18n",
                            "pattern": "^[\\w.-]+$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "[translate]",
                            "attribute": "translate",
                            "pattern": "^(yes|no)$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "time",
                            "attribute": "datetime",
                            "pattern": "^\\d{4}-\\d{2}-\\d{2}(T\\d{2}:\\d{2}(:\\d{2})?)?([+-]\\d{2}:?\\d{2}|Z)?$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "[data-currency]",
                            "attribute": "data-currency",
                            "pattern": "^[A-Z]{3}$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "[data-measurement]",
                            "attribute": "data-measurement",
                            "pattern": "^(metric|imperial)$",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options
            },
        },
        // Image Optimization Compound
        Rule {
            name: "image-optimization-compound".to_string(),
            rule_type: RuleType::Compound,
            severity: Severity::Warning,
            selector: "img".to_string(),
            condition: "all-conditions-met".to_string(),
            message: "Images should implement all modern optimization techniques".to_string(),
            options: {
                let mut options = HashMap::new();
                options.insert("check_mode".to_string(), "all".to_string());
                options.insert(
                    "conditions".to_string(),
                    r#"[
                        {
                            "type": "AttributeValue",
                            "selector": "source[type='image/webp']",
                            "attribute": "srcset",
                            "pattern": ".*\\.webp(\\s+\\d+w)?(,\\s*.*\\.webp(\\s+\\d+w)?)*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "source[type='image/avif']",
                            "attribute": "srcset",
                            "pattern": ".*\\.avif(\\s+\\d+w)?(,\\s*.*\\.avif(\\s+\\d+w)?)*",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "img",
                            "attribute": "loading",
                            "pattern": "^lazy$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "img",
                            "attribute": "decoding",
                            "pattern": "^async$",
                            "check_mode": "ensure_existence"
                        },
                        {
                            "type": "AttributeValue",
                            "selector": "picture, img",
                            "attribute": "sizes",
                            "pattern": "^\\([^)]+\\)\\s+\\d+[vw]px(,\\s*\\([^)]+\\)\\s+\\d+[vw]px)*",
                            "check_mode": "ensure_existence"
                        }
                    ]"#.to_string(),
                );
                options
            },
        }
    ]
}

#[test]
fn test_seo_best_practices() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);
    let html = r#"
        <html>
            <head>
                <title>This is a well-optimized page title for SEO</title>
                <meta name="description" content="This is a comprehensive meta description that provides a good overview of the page content and includes relevant keywords.">
                <link rel="canonical" href="https://example.com/page" />
            </head>
            <body>
                <h1>Main Title</h1>
                <h2>Subtitle</h2>
                <h3>Section</h3>
            </body>
        </html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.len(),
        0,
        "Expected no violations for SEO-optimized page"
    );
}

#[test]
fn test_meta_description_length() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test too short description
    let html = r#"
        <html><head>
            <meta name="description" content="Too short">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "meta-description"),
        "Should detect too short meta description"
    );

    // Test too long description
    let html = r#"
        <html><head>
            <meta name="description" content="This description is way too long and exceeds the recommended maximum length. It contains unnecessary repetition and filler words that don't add value to the description. Search engines will likely truncate this description, making it less effective for SEO purposes. It's better to be concise and meaningful.">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "meta-description"),
        "Should detect too long meta description"
    );
}

#[test]
fn test_title_length() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test too short title
    let html = r#"
        <html><head><title>Too short</title></head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "meta-title"),
        "Should detect too short title"
    );

    // Test too long title
    let html = r#"
        <html><head><title>This title is way too long and will be truncated in search engine results pages</title></head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "meta-title"),
        "Should detect too long title"
    );
}

#[test]
fn test_canonical_url() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing canonical
    let html = r#"
        <html><head><title>Page Title</title></head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "canonical-url"),
        "Should detect missing canonical URL"
    );

    // Test invalid canonical
    let html = r#"
        <html><head>
            <link rel="canonical" />
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "canonical-url"),
        "Should detect invalid canonical URL"
    );
}

#[test]
fn test_heading_hierarchy() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test skipped heading level
    let html = r#"
        <html><body>
            <h1>Title</h1>
            <h3>Skipped H2</h3>
        </body></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "heading-hierarchy"),
        "Should detect skipped heading levels"
    );

    // Test multiple h1 tags
    let html = r#"
        <html><body>
            <h1>First Title</h1>
            <h1>Second Title</h1>
        </body></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "heading-hierarchy"),
        "Should detect multiple H1 tags"
    );
}

#[test]
fn test_complex_seo_scenarios() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test multiple issues
    let html = r#"
        <html>
            <head>
                <title>Short</title>
                <meta name="description" content="Too short">
            </head>
            <body>
                <h2>Missing H1</h2>
                <h4>Skipped H3</h4>
            </body>
        </html>
    "#;
    let results = linter.lint(html).unwrap();

    let violations: Vec<_> = results.iter().map(|r| &r.rule).collect();
    assert!(violations.contains(&&"meta-title".to_string()));
    assert!(violations.contains(&&"meta-description".to_string()));
    assert!(violations.contains(&&"heading-hierarchy".to_string()));
    assert!(violations.contains(&&"canonical-url".to_string()));
}

#[test]
fn test_valid_variations() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test different valid meta description lengths
    let test_cases = vec![
        "This is a meta description that meets the minimum length requirement but isn't too verbose.",
        "This meta description provides comprehensive information about the page content while staying within limits.",
        "A well-crafted meta description that includes relevant keywords and maintains optimal length for SEO.",
    ];

    for description in test_cases {
        let html = format!(
            r#"
            <html>
                <head>
                    <title>This is a well-optimized page title</title>
                    <meta name="description" content="{}">
                    <link rel="canonical" href="https://example.com/page" />
                </head>
                <body>
                    <h1>Main Title</h1>
                    <h2>Subtitle</h2>
                </body>
            </html>
            "#,
            description
        );
        let results = linter.lint(&html).unwrap();
        assert_eq!(
            results.len(),
            0,
            "Valid meta description should not trigger violations"
        );
    }
}

#[test]
fn test_meta_robots() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing robots meta
    let html = r#"<html><head><title>Page</title></head></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "meta-robots"),
        "Should detect missing robots meta tag"
    );

    // Test invalid robots content
    let html = r#"
        <html><head>
            <meta name="robots" content="invalid-directive">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "meta-robots"),
        "Should detect invalid robots directives"
    );

    // Test valid robots combinations
    let valid_contents = vec![
        "index, follow",
        "noindex, follow",
        "index, nofollow",
        "noindex, nofollow",
        "max-snippet:-1, max-image-preview:large",
    ];

    for content in valid_contents {
        let html = format!(
            r#"<html><head>
                <meta name="robots" content="{}">
            </head></html>"#,
            content
        );
        let results = linter.lint(&html).unwrap();
        assert!(
            !results.iter().any(|r| r.rule == "meta-robots"),
            "Should accept valid robots content: {}",
            content
        );
    }
}

#[test]
fn test_open_graph_tags() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing required OG tags
    let html = r#"<html><head><title>Page</title></head></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "og-tags"),
        "Should detect missing Open Graph tags"
    );

    // Test complete OG implementation
    let html = r#"
        <html><head>
            <meta property="og:title" content="Page Title">
            <meta property="og:description" content="A comprehensive description of the page content that provides value to potential visitors.">
            <meta property="og:image" content="https://example.com/image.jpg">
            <meta property="og:url" content="https://example.com/page">
            <meta property="og:type" content="website">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.iter().filter(|r| r.rule == "og-tags").count(),
        0,
        "Should accept complete Open Graph implementation"
    );
}

#[test]
fn test_twitter_cards() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing Twitter card tags
    let html = r#"<html><head><title>Page</title></head></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "twitter-cards"),
        "Should detect missing Twitter card tags"
    );

    // Test valid Twitter card implementation
    let html = r#"
        <html><head>
            <meta name="twitter:card" content="summary_large_image">
            <meta name="twitter:title" content="Page Title">
            <meta name="twitter:description" content="A comprehensive description of the page content optimized for Twitter sharing.">
            <meta name="twitter:image" content="https://example.com/image.jpg">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.iter().filter(|r| r.rule == "twitter-cards").count(),
        0,
        "Should accept valid Twitter card implementation"
    );
}

#[test]
fn test_structured_data() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing structured data
    let html = r#"<html><head><title>Page</title></head></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "structured-data"),
        "Should detect missing structured data"
    );

    // Test invalid JSON-LD
    let html = r#"
        <html><head>
            <script type="application/ld+json">
                { "invalid": "json" "schema" }
            </script>
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "structured-data"),
        "Should detect invalid JSON-LD syntax"
    );

    // Test valid structured data
    let html = r#"
        <html><head>
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "Article",
                "headline": "Article Title",
                "author": {
                    "@type": "Person",
                    "name": "John Doe"
                },
                "datePublished": "2023-01-01",
                "description": "Article description"
            }
            </script>
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results
            .iter()
            .filter(|r| r.rule == "structured-data")
            .count(),
        0,
        "Should accept valid structured data"
    );
}

#[test]
fn test_pagination_tags() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing pagination tags on paginated content
    let html = r#"
        <html><head>
            <title>Page 2 of Articles</title>
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "pagination-tags"),
        "Should detect missing pagination tags"
    );

    // Test valid pagination implementation
    let html = r#"
        <html><head>
            <title>Page 2 of Articles</title>
            <link rel="canonical" href="https://example.com/articles/page/2">
            <link rel="prev" href="https://example.com/articles/page/1">
            <link rel="next" href="https://example.com/articles/page/3">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results
            .iter()
            .filter(|r| r.rule == "pagination-tags")
            .count(),
        0,
        "Should accept valid pagination tags"
    );
}

#[test]
fn test_hreflang_tags() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing hreflang on multi-language site
    let html = r#"
        <html lang="en"><head>
            <title>English Page</title>
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "hreflang-tags"),
        "Should detect missing hreflang tags"
    );

    // Test valid hreflang implementation
    let html = r#"
        <html lang="en"><head>
            <title>English Page</title>
            <link rel="alternate" hreflang="en" href="https://example.com/page">
            <link rel="alternate" hreflang="es" href="https://example.com/es/page">
            <link rel="alternate" hreflang="fr" href="https://example.com/fr/page">
            <link rel="alternate" hreflang="x-default" href="https://example.com/page">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.iter().filter(|r| r.rule == "hreflang-tags").count(),
        0,
        "Should accept valid hreflang implementation"
    );
}

#[test]
fn test_mobile_viewport() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test missing viewport meta
    let html = r#"<html><head><title>Page</title></head></html>"#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "viewport-meta"),
        "Should detect missing viewport meta tag"
    );

    // Test invalid viewport content
    let html = r#"
        <html><head>
            <meta name="viewport" content="width=1024">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results.iter().any(|r| r.rule == "viewport-meta"),
        "Should detect non-responsive viewport settings"
    );

    // Test valid viewport
    let html = r#"
        <html><head>
            <meta name="viewport" content="width=device-width, initial-scale=1">
        </head></html>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results.iter().filter(|r| r.rule == "viewport-meta").count(),
        0,
        "Should accept valid viewport meta tag"
    );
}

#[test]
fn test_image_optimization_compound() {
    let linter = HtmlLinter::new(setup_seo_rules(), None);

    // Test case with all optimizations
    let html = r#"
        <picture>
            <source 
                type="image/avif" 
                srcset="image.avif 1x, image@2x.avif 2x"
                sizes="(max-width: 768px) 100vw, 50vw"
            />
            <source 
                type="image/webp" 
                srcset="image.webp 1x, image@2x.webp 2x"
                sizes="(max-width: 768px) 100vw, 50vw"
            />
            <img 
                src="image.jpg" 
                loading="lazy"
                decoding="async"
                sizes="(max-width: 768px) 100vw, 50vw"
                alt="Optimized image"
            />
        </picture>
    "#;
    let results = linter.lint(html).unwrap();
    assert_eq!(
        results
            .iter()
            .filter(|r| r.rule == "image-optimization-compound")
            .count(),
        0,
        "Should pass when all optimizations are present"
    );

    // Test case missing optimizations
    let html = r#"
        <img src="image.jpg" alt="Non-optimized image">
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results
            .iter()
            .any(|r| r.rule == "image-optimization-compound"),
        "Should fail when optimizations are missing"
    );

    // Test case with partial optimizations
    let html = r#"
        <picture>
            <source 
                type="image/webp" 
                srcset="image.webp"
            />
            <img 
                src="image.jpg" 
                loading="lazy"
                alt="Partially optimized image"
            />
        </picture>
    "#;
    let results = linter.lint(html).unwrap();
    assert!(
        results
            .iter()
            .any(|r| r.rule == "image-optimization-compound"),
        "Should fail when some optimizations are missing"
    );
}
