use crate::dom::utils::extract_text;
use crate::*;

impl HtmlLinter {
    pub(crate) fn check_custom(
        &self,
        rule: &Rule,
        validator: &str,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let (should_report, detailed_message) = match validator {
                    "no-empty-links" => {
                        let is_link =
                            index.resolve_symbol(node.tag_name).unwrap_or_default() == "a";
                        let is_empty = node.children.is_empty();
                        (
                            is_link && is_empty,
                            "Link element has no content. Links should contain text or other content to describe their purpose".to_string(),
                        )
                    }
                    "no-empty-headings" => {
                        let tag_name = index.resolve_symbol(node.tag_name).unwrap_or_default();
                        let is_heading = tag_name.starts_with('h');
                        let is_empty = node.children.is_empty();
                        (
                            is_heading && is_empty,
                            format!("Heading element <{}> has no content. Headings should contain text to maintain document structure", tag_name),
                        )
                    }
                    _ => (false, String::new()),
                };

                if should_report {
                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: format!("{} - {}", rule.message, detailed_message),
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: index
                                .resolve_symbol(node.tag_name)
                                .unwrap_or_default()
                                .to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    pub(crate) fn check_compound(
        &self,
        rule: &Rule,
        index: &DOMIndex,
    ) -> Result<Vec<LintResult>, LinterError> {
        let mut results = Vec::new();
        let matches = index.query(&rule.selector);

        let conditions: Vec<CompoundCondition> = rule
            .options
            .get("conditions")
            .ok_or_else(|| {
                LinterError::RuleError("Missing conditions for compound rule".to_string())
            })
            .and_then(|conditions_str| {
                serde_json::from_str(conditions_str)
                    .map_err(|e| LinterError::RuleError(format!("Invalid conditions JSON: {}", e)))
            })?;

        let check_mode = rule
            .options
            .get("check_mode")
            .map(String::as_str)
            .unwrap_or("all");

        for node_idx in matches {
            if let Some(node) = index.get_node(node_idx) {
                let matching_conditions: Vec<bool> = conditions
                    .iter()
                    .map(|condition| self.check_single_condition(condition, node_idx, index))
                    .collect();

                let should_report = match check_mode {
                    "any" => !matching_conditions.iter().any(|&x| x),
                    "all" => !matching_conditions.iter().all(|&x| x),
                    "none" => matching_conditions.iter().any(|&x| x),
                    "exactly_one" => matching_conditions.iter().filter(|&&x| x).count() != 1,
                    "at_least_one" => !matching_conditions.iter().any(|&x| x),
                    "majority" => {
                        let count = matching_conditions.iter().filter(|&&x| x).count();
                        count <= conditions.len() / 2
                    }
                    "weighted" => {
                        let weights = rule
                            .options
                            .get("weights")
                            .and_then(|w| serde_json::from_str::<Vec<f64>>(w).ok())
                            .unwrap_or_else(|| vec![1.0; conditions.len()]);
                        let threshold = rule
                            .options
                            .get("threshold")
                            .and_then(|t| t.parse::<f64>().ok())
                            .unwrap_or(1.0);

                        let total_weight = matching_conditions
                            .iter()
                            .zip(weights.iter())
                            .filter_map(
                                |(&matched, &weight)| if matched { Some(weight) } else { None },
                            )
                            .sum::<f64>();

                        total_weight < threshold
                    }
                    "dependency_chain" => {
                        // Check if there's a gap between any matching conditions
                        // Example: [true, true, false, true] -> should report error
                        //          [true, true, true, false] -> should not report error
                        let first_false = matching_conditions.iter().position(|&x| !x);
                        let any_true_after = first_false
                            .map(|pos| matching_conditions[pos..].iter().any(|&x| x))
                            .unwrap_or(false);

                        any_true_after
                    }
                    "alternating" => matching_conditions.windows(2).any(|w| w[0] == w[1]),
                    "subset_match" => {
                        if let Some(valid_sets_str) = rule.options.get("valid_sets") {
                            if let Ok(valid_sets) =
                                serde_json::from_str::<Vec<Vec<usize>>>(valid_sets_str)
                            {
                                let current_set: Vec<usize> = matching_conditions
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, &matched)| matched)
                                    .map(|(i, _)| i)
                                    .collect();
                                !valid_sets.iter().any(|set| {
                                    set.iter().all(|&idx| current_set.contains(&idx))
                                        && current_set.iter().all(|&idx| set.contains(&idx))
                                })
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if should_report {
                    let matching_count = matching_conditions.iter().filter(|&&x| x).count();
                    let total_conditions = conditions.len();

                    let detailed_message = match check_mode {
                        "any" => format!(
                            "None of the {} conditions were met. At least one condition must be satisfied",
                            total_conditions
                        ),
                        "all" => format!(
                            "Only {}/{} conditions were satisfied. All conditions must be met",
                            matching_count,
                            total_conditions
                        ),
                        "none" => format!(
                            "Found {} matching conditions where none should match. All conditions must fail",
                            matching_count
                        ),
                        "exactly_one" => format!(
                            "Found {} matching conditions where exactly 1 was expected",
                            matching_count
                        ),
                        "at_least_one" => format!(
                            "Found no matching conditions. At least 1 of {} conditions must match",
                            total_conditions
                        ),
                        "majority" => format!(
                            "Only {}/{} conditions matched. More than half ({}) must match",
                            matching_count,
                            total_conditions,
                            (total_conditions / 2) + 1
                        ),
                        "weighted" => {
                            let weights = rule
                                .options
                                .get("weights")
                                .and_then(|w| serde_json::from_str::<Vec<f64>>(w).ok())
                                .unwrap_or_else(|| vec![1.0; total_conditions]);
                            let threshold = rule
                                .options
                                .get("threshold")
                                .and_then(|t| t.parse::<f64>().ok())
                                .unwrap_or(1.0);
                            let total_weight: f64 = matching_conditions
                                .iter()
                                .zip(weights.iter())
                                .filter_map(|(&matched, &weight)| if matched { Some(weight) } else { None })
                                .sum();
                            format!(
                                "Total weight of matching conditions ({:.2}) is below required threshold ({:.2})",
                                total_weight,
                                threshold
                            )
                        },
                        "dependency_chain" => {
                            let chain_length = matching_conditions.iter().take_while(|&&x| x).count();
                            format!(
                                "Chain broken after {} conditions. Expected unbroken chain of {} matching conditions",
                                chain_length,
                                matching_count
                            )
                        },
                        "alternating" => {
                            let violation_index = matching_conditions
                                .windows(2)
                                .position(|w| w[0] == w[1])
                                .map(|i| i + 1)
                                .unwrap_or(0);
                            format!(
                                "Found consecutive {} conditions at position {}. Pattern must alternate between match/no-match",
                                if matching_conditions[violation_index] { "matching" } else { "non-matching" },
                                violation_index + 1
                            )
                        },
                        "subset_match" => {
                            if let Some(valid_sets_str) = rule.options.get("valid_sets") {
                                if let Ok(valid_sets) = serde_json::from_str::<Vec<Vec<usize>>>(valid_sets_str) {
                                    let current_set: Vec<usize> = matching_conditions
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, &matched)| matched)
                                        .map(|(i, _)| i)
                                        .collect();
                                    format!(
                                        "Current matching set {:?} doesn't match any valid combination. Valid sets: {:?}",
                                        current_set,
                                        valid_sets
                                    )
                                } else {
                                    "Invalid valid_sets configuration".to_string()
                                }
                            } else {
                                "Missing valid_sets configuration".to_string()
                            }
                        },
                        _ => "Compound condition check failed".to_string(),
                    };

                    let condition_details: Vec<String> = conditions
                        .iter()
                        .zip(matching_conditions.iter())
                        .map(|(condition, &matched)| {
                            let status = if matched { "✓" } else { "✗" };
                            match condition {
                                CompoundCondition::TextContent { pattern } => {
                                    format!("{} Text content pattern '{}' match", status, pattern)
                                }
                                CompoundCondition::AttributeValue { attribute, pattern } => {
                                    format!(
                                        "{} Attribute '{}' matching pattern '{}'",
                                        status, attribute, pattern
                                    )
                                }
                                CompoundCondition::AttributeReference {
                                    attribute,
                                    reference_must_exist,
                                } => format!(
                                    "{} Attribute '{}' reference {}",
                                    status,
                                    attribute,
                                    if *reference_must_exist {
                                        "exists"
                                    } else {
                                        "does not exist"
                                    }
                                ),
                                CompoundCondition::ElementPresence { selector } => {
                                    format!(
                                        "{} Element presence {}",
                                        status,
                                        if matched { "exists" } else { "does not exist" }
                                    )
                                }
                            }
                        })
                        .collect();

                    results.push(LintResult {
                        rule: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: format!(
                            "{} - {} \nCondition details:\n{}",
                            rule.message,
                            detailed_message,
                            condition_details.join("\n")
                        ),
                        location: Location {
                            line: node.source_info.line,
                            column: node.source_info.column,
                            element: index
                                .resolve_symbol(node.tag_name)
                                .unwrap_or_default()
                                .to_string(),
                        },
                        source: node.source_info.source.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    fn check_single_condition(
        &self,
        condition: &CompoundCondition,
        node_idx: usize,
        index: &DOMIndex,
    ) -> bool {
        match condition {
            CompoundCondition::TextContent { pattern } => {
                let node = index.get_node(node_idx).unwrap();
                let mut content = String::new();
                if let Some(handle) = &node.handle {
                    extract_text(handle, &mut content);
                    if content.trim().is_empty() {
                        return false;
                    }
                    Regex::new(pattern)
                        .map(|regex| regex.is_match(content.trim()))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            CompoundCondition::AttributeValue { attribute, pattern } => {
                if let Some(node) = index.get_node(node_idx) {
                    if let Ok(regex) = Regex::new(pattern) {
                        return node.attributes.iter().any(|attr| {
                            let name = index.resolve_symbol(attr.name).unwrap_or_default();
                            let value = index.resolve_symbol(attr.value).unwrap_or_default();
                            name == *attribute
                                && !value.trim().is_empty()
                                && regex.is_match(value.trim())
                        });
                    }
                }
                false
            }
            CompoundCondition::AttributeReference {
                attribute,
                reference_must_exist,
            } => {
                if let Some(node) = index.get_node(node_idx) {
                    if let Some(attr) = node.attributes.iter().find(|attr| {
                        index.resolve_symbol(attr.name).unwrap_or_default() == *attribute
                    }) {
                        let value = index.resolve_symbol(attr.value).unwrap_or_default();
                        if !value.trim().is_empty() {
                            let referenced_selector = format!("[id=\"{}\"]", value.trim());
                            let exists = !index.query(&referenced_selector).is_empty();
                            return exists == *reference_must_exist;
                        }
                    }
                }
                false
            }
            CompoundCondition::ElementPresence { selector } => {
                // Check if any elements matching the selector exist within the current node's scope
                if let Some(_node) = index.get_node(node_idx) {
                    // Create a scoped selector by prepending the current node's selector
                    let current_selector = format!("{} {}", selector, selector);
                    let matches = index.query(&current_selector);
                    !matches.is_empty()
                } else {
                    false
                }
            }
        }
    }
}
