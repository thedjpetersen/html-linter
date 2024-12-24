# html-linter

[![Crates.io](https://img.shields.io/crates/v/html-linter.svg)](https://crates.io/crates/html-linter)
[![Documentation](https://docs.rs/html-linter/badge.svg)](https://docs.rs/html-linter)
[![License](https://img.shields.io/crates/l/html-linter.svg)](LICENSE)

<p align="center">
  <img src="docs/images/logo.png" alt="HTML Linter Logo" width="200">
</p>

`html-linter` is a Rust library for linting HTML content. It checks for various common errors, best practices, semantic issues, and more. You can supply rules that specify what to check (element presence, attribute presence, ordering constraints, semantic constraints, etc.) and how severe each issue should be when found.

## Features

- **Rule-based**: Each linting requirement is defined as a `Rule` (e.g., "Images must have alt attributes").
- **Multiple rule types**: Check everything from element presence to whitespace, attribute values, heading order, nesting, and more.
- **Customizable**: You can define your own rules, including custom selectors and even custom logic.
- **Semantic checks**: Lint for semantic HTML usage (e.g., using `<header>` instead of `<div class="header">`).
- **SEO checks**: Validate required meta tags, Open Graph tags, etc.
- **Configurable**: Customize options such as maximum line length and ignoring inline styles.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
html-linter = "0.1.0"
```

And then bring it into scope in your code:

```rust
use html_linter::{HtmlLinter, LinterOptions, Rule, RuleType, Severity, LintResult};
```

## Usage

### 1. Define your linting rules

You can define rules either programmatically or using JSON configuration:

#### Option A: Programmatic Rule Definition

```rust
use std::collections::HashMap;
use html_linter::{Rule, RuleType, Severity};

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
];
```

#### Option B: JSON Configuration

```json
[
  {
    "name": "img-alt",
    "rule_type": "AttributePresence",
    "severity": "Error",
    "selector": "img",
    "condition": "alt-missing",
    "message": "Images must have alt attributes",
    "options": {}
  },
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
  }
]
```

You can load JSON rules either from a string or from a file:

```rust
// Load from JSON string
let linter = HtmlLinter::from_json(json_str, None)?;

// Load from JSON file
let linter = HtmlLinter::from_json_file("path/to/rules.json", None)?;
```

### JSON Rule Configuration Reference

Each rule in the JSON configuration must follow this structure:

```json
{
  "name": "string", // Unique identifier for the rule
  "rule_type": "string", // One of the supported rule types
  "severity": "string", // "Error", "Warning", or "Info"
  "selector": "string", // CSS-like selector
  "condition": "string", // Rule-specific condition
  "message": "string", // Error message to display
  "options": {
    // Optional additional configuration
    "key": "value"
  }
}
```

#### Supported Rule Types

1. **ElementPresence**

   ```json
   {
     "name": "require-main",
     "rule_type": "ElementPresence",
     "severity": "Error",
     "selector": "main",
     "condition": "required",
     "message": "Page must have a main content area"
   }
   ```

2. **AttributePresence**

   ```json
   {
     "name": "img-alt",
     "rule_type": "AttributePresence",
     "severity": "Error",
     "selector": "img",
     "condition": "alt-missing",
     "message": "Images must have alt attributes"
   }
   ```

3. **AttributeValue**

   ```json
   {
     "name": "valid-email",
     "rule_type": "AttributeValue",
     "selector": "input[type='email']",
     "severity": "Error",
     "condition": "pattern-match",
     "message": "Invalid email pattern",
     "options": {
       "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
     }
   }
   ```

4. **ElementOrder**

   ```json
   {
     "name": "heading-order",
     "rule_type": "ElementOrder",
     "severity": "Warning",
     "selector": "h1, h2, h3, h4, h5, h6",
     "condition": "sequential-order",
     "message": "Heading levels should not skip levels"
   }
   ```

5. **ElementContent**

   ```json
   {
     "name": "meta-description",
     "rule_type": "ElementContent",
     "severity": "Error",
     "selector": "head",
     "condition": "meta-tags",
     "message": "Required meta tags are missing",
     "options": {
       "required_meta_tags": [
         {
           "name": "description",
           "pattern": {
             "type": "MinLength",
             "value": 50
           },
           "required": true
         }
       ]
     }
   }
   ```

6. **WhiteSpace**

   ```json
   {
     "name": "line-length",
     "rule_type": "WhiteSpace",
     "severity": "Warning",
     "selector": "*",
     "condition": "line-length",
     "message": "Line exceeds maximum length",
     "options": {
       "max_line_length": "80"
     }
   }
   ```

7. **Nesting**

   ```json
   {
     "name": "input-label",
     "rule_type": "Nesting",
     "severity": "Error",
     "selector": "input",
     "condition": "parent-label-or-for",
     "message": "Input elements must be associated with a label"
   }
   ```

8. **Semantics**

   ```json
   {
     "name": "semantic-html",
     "rule_type": "Semantics",
     "severity": "Warning",
     "selector": "div",
     "condition": "semantic-structure",
     "message": "Use semantic HTML elements instead of divs where appropriate"
   }
   ```

9. **Custom**
   ```json
   {
     "name": "no-empty-links",
     "rule_type": "Custom",
     "severity": "Error",
     "selector": "a",
     "condition": "no-empty-links",
     "message": "Links must have content"
   }
   ```

#### Meta Tag Patterns

When using `ElementContent` with `meta-tags`, the following pattern types are supported:

- `Regex`: Match content against a regular expression
- `MinLength`: Require minimum character length
- `MaxLength`: Limit maximum character length
- `NonEmpty`: Ensure content is not empty
- `Exact`: Match exact text
- `OneOf`: Match one of several options
- `Contains`: Check if content contains substring
- `StartsWith`: Check if content starts with string
- `EndsWith`: Check if content ends with string

Example meta tag pattern:

```json
{
  "pattern": {
    "type": "MinLength",
    "value": 50
  }
}
```

### 2. Create an `HtmlLinter`

```rust
use html_linter::{HtmlLinter, LinterOptions};

// Optional: specify linter-wide options (e.g., max line length, ignoring inline styles, etc.)
let options = LinterOptions {
    // For example, ignore lines longer than 80 characters
    max_line_length: Some(80),
    // ...other options...
    ..Default::default()
};

// Build the linter with your rules and options
let linter = HtmlLinter::new(rules, Some(options));
```

### 3. Lint HTML content

```rust
let html = r#"<html><body><img src="test.jpg"></body></html>"#;

// Lint returns a Result containing either a vector of `LintResult` or a `LinterError`.
let lint_results = linter.lint(html).unwrap();

// Each `LintResult` contains:
// - the triggered rule's name
// - the severity level
// - a descriptive message
// - location info (line, column, and element name)
// - partial source snippet of the element
```

### Example

```rust
fn main() {
    // Define rules
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
    ];

    // Create the linter
    let linter = HtmlLinter::new(rules, None);

    // Some HTML to check
    let html = r#"<html><body><img src="test.jpg"></body></html>"#;

    // Perform linting
    match linter.lint(html) {
        Ok(results) => {
            for result in results {
                println!(
                    "Rule: {}, Severity: {:?}, Message: {}, Location: line {}, column {}",
                    result.rule,
                    result.severity,
                    result.message,
                    result.location.line,
                    result.location.column
                );
            }
        }
        Err(e) => eprintln!("Linter error: {}", e),
    }
}
```

### Writing Tests

By default, if you have a `tests/` directory in your crate, you can add integration tests for your linting rules. See the [examples in the code above](./tests/) to learn how to organize them. Run them with:

```bash
cargo test
```

## Rule Types

The library supports several `RuleType`s, each controlling how the rule is evaluated:

- **`ElementPresence`**: Checks if certain elements exist (or do not exist).
- **`AttributePresence`**: Checks if specific attributes are present (or missing).
- **`AttributeValue`**: Validates attribute values against a regex or other criteria.
- **`ElementOrder`**: Ensures elements follow a certain order (e.g., heading levels).
- **`ElementContent`**: Validates text content or checks for empty content.
- **`WhiteSpace`**: Checks whitespace and line length constraints.
- **`Nesting`**: Ensures certain elements are nested within parent elements or properly associated.
- **`Semantics`**: Encourages semantic HTML usage (e.g., `<header>` instead of `<div class="header">`).
- **`Custom(String)`**: Custom rule logic with a built-in function key (e.g., `"no-empty-links"`).

## Contributing

Pull requests, bug reports, and feature requests are welcome! Feel free to open an issue or submit a PR if you have ideas to improve the library.

## License

This project is licensed under the [MIT license](./LICENSE).
Enjoy responsibly.

```

```
