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

### ElementPresence

Checks if certain elements exist (or do not exist).

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

### AttributePresence

Checks if specific attributes are present (or missing).

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

### AttributeValue

Validates attribute values against a regex or other criteria.

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

### ElementOrder

Ensures elements follow a certain order (e.g., heading levels).

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

### ElementContent

Validates text content or checks for empty content.

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

### WhiteSpace

**Not implemented**

### Nesting

Ensures certain elements are nested within parent elements or properly associated.

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

### Semantics

Encourages semantic HTML usage (e.g., `<header>` instead of `<div class="header">`).

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

### Custom

Custom rule logic with a built-in function key (e.g., `"no-empty-links"`).

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

### Compound

Allows combining multiple conditions that must all be satisfied. Supports various check modes and condition types.

```json
{
  "name": "accessible-button",
  "rule_type": "Compound",
  "severity": "Error",
  "selector": "button",
  "condition": "compound",
  "message": "Button must meet accessibility requirements",
  "options": {
    "check_mode": "all",
    "conditions": [
      {
        "type": "AttributeValue",
        "attribute": "aria-label",
        "pattern": ".+"
      },
      {
        "type": "AttributeValue",
        "attribute": "role",
        "pattern": "button"
      }
    ]
  }
}
```

#### Compound Rule Check Modes

- `all`: All conditions must match (default)
- `any`: Any condition must match
- `none`: No conditions should match
- `exactly_one`: Exactly one condition should match
- `at_least_one`: At least one condition must match
- `majority`: More than half of conditions must match
- `ratio`: Specified ratio of conditions must match (requires "ratio" option)
- `range`: Number of matching conditions must fall within specified range (requires "min" and "max" options)
- `consecutive`: Specified number of consecutive conditions must match (requires "count" option)
- `exclusive_groups`: Only one group of conditions should match (requires "groups" option)
- `weighted`: Sum of weights for matching conditions must meet threshold (requires "weights" and "threshold" options)
- `dependency_chain`: Conditions must match in sequence without gaps
- `alternating`: Conditions must alternate between matching and non-matching
- `subset_match`: Matching conditions must form a valid subset (requires "valid_sets" option)

Example with advanced check mode:

```json
{
  "name": "weighted-conditions",
  "rule_type": "Compound",
  "severity": "Error",
  "selector": "form",
  "condition": "compound",
  "message": "Form must meet weighted accessibility requirements",
  "options": {
    "check_mode": "weighted",
    "weights": [0.5, 1.0, 0.8],
    "threshold": 1.5,
    "conditions": [
      {
        "type": "AttributeValue",
        "attribute": "aria-label",
        "pattern": ".+"
      },
      {
        "type": "AttributeValue",
        "attribute": "role",
        "pattern": "form"
      },
      {
        "type": "AttributeValue",
        "attribute": "name",
        "pattern": ".+"
      }
    ]
  }
}
```

#### Compound Rule Condition Types

Compound rules support three types of conditions:

1. **TextContent**

```json
{
  "type": "TextContent",
  "pattern": "^[A-Za-z0-9\\s]{10,}$"
}
```

2. **AttributeValue**

```json
{
  "type": "AttributeValue",
  "attribute": "class",
  "pattern": "^btn-[a-z]+$"
}
```

3. **AttributeReference**

```json
{
  "type": "AttributeReference",
  "attribute": "aria-describedby",
  "reference_must_exist": true
}
```

### TextContent

Validates the text content of elements against patterns.

```json
{
  "name": "min-heading-length",
  "rule_type": "TextContent",
  "severity": "Warning",
  "selector": "h1, h2, h3",
  "condition": "text-content",
  "message": "Heading text should be descriptive",
  "options": {
    "pattern": ".{10,}",
    "check_mode": "ensure_existence"
  }
}
```

### Pattern Types for Content Validation

When validating content (especially with `TextContent` or `ElementContent`), the following pattern types are supported:

- `Regex`: Match content against a regular expression
- `MinLength`: Require minimum character length
- `MaxLength`: Limit maximum character length
- `NonEmpty`: Ensure content is not empty
- `Exact`: Match exact text
- `OneOf`: Match one of several options
- `Contains`: Check if content contains substring
- `StartsWith`: Check if content starts with string
- `EndsWith`: Check if content ends with string

Example using different pattern types:

```json
{
  "name": "meta-tags",
  "rule_type": "ElementContent",
  "severity": "Error",
  "selector": "head",
  "condition": "meta-tags",
  "message": "Meta tags validation failed",
  "options": {
    "required_meta_tags": [
      {
        "name": "description",
        "pattern": {
          "type": "MinLength",
          "value": 50
        },
        "required": true
      },
      {
        "name": "keywords",
        "pattern": {
          "type": "Contains",
          "value": "important-keyword"
        },
        "required": true
      },
      {
        "property": "og:type",
        "pattern": {
          "type": "OneOf",
          "value": ["website", "article", "product"]
        },
        "required": true
      }
    ]
  }
}
```

### Check Modes

Many rule types support different check modes that modify how the rule is evaluated:

- `normal`: Default behavior - report when pattern matches
- `ensure_existence`: Report when pattern doesn't match (inverse)
- `ensure_nonexistence`: Report when pattern matches (same as normal)
- `any`: For compound rules - any condition must match
- `all`: For compound rules - all conditions must match

Example using check modes:

```json
{
  "name": "no-placeholder-images",
  "rule_type": "AttributeValue",
  "severity": "Warning",
  "selector": "img",
  "condition": "src-check",
  "message": "Avoid using placeholder image services",
  "options": {
    "check_mode": "ensure_nonexistence",
    "pattern": "placeholder\\.com|placekitten\\.com"
  }
}
```

### DocumentStructure

Validates the overall structure of the HTML document.

```json
{
  "name": "require-doctype",
  "rule_type": "DocumentStructure",
  "severity": "Error",
  "selector": "html",
  "condition": "doctype-present",
  "message": "HTML document must have a DOCTYPE declaration"
}
```

### ElementCount

Enforces limits on the number of specific elements.

```json
{
  "name": "single-h1",
  "rule_type": "ElementCount",
  "severity": "Error",
  "selector": "h1",
  "condition": "max-count",
  "message": "Page should have only one h1 element",
  "options": {
    "max": "1"
  }
}
```

### ElementCase

Enforces consistent casing for element and attribute names.

```json
{
  "name": "lowercase-elements",
  "rule_type": "ElementCase",
  "severity": "Warning",
  "selector": "*",
  "condition": "lowercase",
  "message": "HTML elements and attributes should be lowercase",
  "options": {}
}
```

### AttributeQuotes

Enforces consistent use of single or double quotes for attribute values.

```json
{
  "name": "quote-style",
  "rule_type": "AttributeQuotes",
  "severity": "Warning",
  "selector": "*",
  "condition": "quote-style",
  "message": "Use double quotes for attribute values",
  "options": {
    "style": "double"
  }
}
```

## Contributing

Pull requests, bug reports, and feature requests are welcome! Feel free to open an issue or submit a PR if you have ideas to improve the library.

## License

This project is licensed under the [MIT license](./LICENSE).
Enjoy responsibly.
