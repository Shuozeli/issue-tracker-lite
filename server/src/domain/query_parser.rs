/// Query parser for issue search.
///
/// Syntax:
/// - `field:value` for structured filters
/// - `-field:value` for negation
/// - `"exact phrase"` for keyword phrases
/// - bare words for keyword search
///
/// Supported fields: status, priority, severity, type, assignee, reporter,
/// componentid (with `+` suffix for recursive), hotlistid
///
/// Special values: open, closed, none, any

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    Equals,
    NotEquals,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterField {
    Status,
    Priority,
    Severity,
    IssueType,
    Assignee,
    Reporter,
    ComponentId,
    /// ComponentId with recursive flag (componentid:5+)
    ComponentIdRecursive,
    HotlistId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldFilter {
    pub field: FilterField,
    pub op: FilterOp,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuery {
    pub filters: Vec<FieldFilter>,
    pub keywords: Vec<String>,
}

/// Parse a search query string into structured filters and keywords.
pub fn parse_query(input: &str) -> ParsedQuery {
    let mut filters = Vec::new();
    let mut keywords = Vec::new();

    let mut pos = 0;

    while pos < input.len() {
        // Skip whitespace
        while pos < input.len() && input.as_bytes()[pos] == b' ' {
            pos += 1;
        }
        if pos >= input.len() {
            break;
        }

        // Check for negation
        let negated = input.as_bytes()[pos] == b'-'
            && pos + 1 < input.len()
            && input.as_bytes()[pos + 1] != b' ';
        if negated {
            pos += 1;
        }

        // Check for quoted string
        if pos < input.len() && input.as_bytes()[pos] == b'"' {
            pos += 1;
            let start = pos;
            while pos < input.len() && input.as_bytes()[pos] != b'"' {
                pos += 1;
            }
            let phrase = &input[start..pos];
            if pos < input.len() {
                pos += 1; // skip closing quote
            }
            if !phrase.is_empty() {
                keywords.push(phrase.to_string());
            }
            continue;
        }

        // Read token
        let start = pos;
        while pos < input.len() && input.as_bytes()[pos] != b' ' {
            pos += 1;
        }
        let token = &input[start..pos];

        // Check if it's a field:value pair
        if let Some(colon_pos) = token.find(':') {
            let field_str = &token[..colon_pos];
            let value = &token[colon_pos + 1..];

            let op = if negated {
                FilterOp::NotEquals
            } else {
                FilterOp::Equals
            };

            match field_str.to_lowercase().as_str() {
                "status" => {
                    filters.push(FieldFilter {
                        field: FilterField::Status,
                        op,
                        value: value.to_string(),
                    });
                }
                "priority" => {
                    filters.push(FieldFilter {
                        field: FilterField::Priority,
                        op,
                        value: value.to_string(),
                    });
                }
                "severity" => {
                    filters.push(FieldFilter {
                        field: FilterField::Severity,
                        op,
                        value: value.to_string(),
                    });
                }
                "type" => {
                    filters.push(FieldFilter {
                        field: FilterField::IssueType,
                        op,
                        value: value.to_string(),
                    });
                }
                "assignee" => {
                    filters.push(FieldFilter {
                        field: FilterField::Assignee,
                        op,
                        value: value.to_string(),
                    });
                }
                "reporter" => {
                    filters.push(FieldFilter {
                        field: FilterField::Reporter,
                        op,
                        value: value.to_string(),
                    });
                }
                "componentid" => {
                    if value.ends_with('+') {
                        filters.push(FieldFilter {
                            field: FilterField::ComponentIdRecursive,
                            op,
                            value: value.trim_end_matches('+').to_string(),
                        });
                    } else {
                        filters.push(FieldFilter {
                            field: FilterField::ComponentId,
                            op,
                            value: value.to_string(),
                        });
                    }
                }
                "hotlistid" => {
                    filters.push(FieldFilter {
                        field: FilterField::HotlistId,
                        op,
                        value: value.to_string(),
                    });
                }
                _ => {
                    // Unknown field, treat as keyword
                    let full = if negated {
                        format!("-{}", token)
                    } else {
                        token.to_string()
                    };
                    keywords.push(full);
                }
            }
        } else {
            // Bare word -> keyword
            let full = if negated {
                format!("-{}", token)
            } else {
                token.to_string()
            };
            keywords.push(full);
        }
    }

    ParsedQuery { filters, keywords }
}

/// Resolve special status values: "open" -> list of open statuses, "closed" -> list of closed
pub fn resolve_status_value(value: &str) -> Vec<String> {
    match value.to_lowercase().as_str() {
        "open" => vec![
            "NEW".to_string(),
            "ASSIGNED".to_string(),
            "IN_PROGRESS".to_string(),
            "INACTIVE".to_string(),
        ],
        "closed" => vec![
            "FIXED".to_string(),
            "FIXED_VERIFIED".to_string(),
            "WONT_FIX_INFEASIBLE".to_string(),
            "WONT_FIX_NOT_REPRODUCIBLE".to_string(),
            "WONT_FIX_OBSOLETE".to_string(),
            "WONT_FIX_INTENDED_BEHAVIOR".to_string(),
            "DUPLICATE".to_string(),
        ],
        other => vec![other.to_uppercase()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_field_value() {
        let q = parse_query("status:open");
        assert_eq!(q.filters.len(), 1);
        assert_eq!(q.filters[0].field, FilterField::Status);
        assert_eq!(q.filters[0].op, FilterOp::Equals);
        assert_eq!(q.filters[0].value, "open");
        assert!(q.keywords.is_empty());
    }

    #[test]
    fn test_parse_negation() {
        let q = parse_query("-status:closed");
        assert_eq!(q.filters.len(), 1);
        assert_eq!(q.filters[0].op, FilterOp::NotEquals);
        assert_eq!(q.filters[0].value, "closed");
    }

    #[test]
    fn test_parse_keyword() {
        let q = parse_query("memory leak");
        assert!(q.filters.is_empty());
        assert_eq!(q.keywords, vec!["memory", "leak"]);
    }

    #[test]
    fn test_parse_quoted_phrase() {
        let q = parse_query("\"memory leak\"");
        assert!(q.filters.is_empty());
        assert_eq!(q.keywords, vec!["memory leak"]);
    }

    #[test]
    fn test_parse_combined() {
        let q = parse_query("priority:P0 status:open memory leak");
        assert_eq!(q.filters.len(), 2);
        assert_eq!(q.filters[0].field, FilterField::Priority);
        assert_eq!(q.filters[1].field, FilterField::Status);
        assert_eq!(q.keywords, vec!["memory", "leak"]);
    }

    #[test]
    fn test_parse_component_recursive() {
        let q = parse_query("componentid:5+");
        assert_eq!(q.filters.len(), 1);
        assert_eq!(q.filters[0].field, FilterField::ComponentIdRecursive);
        assert_eq!(q.filters[0].value, "5");
    }

    #[test]
    fn test_resolve_status_open() {
        let statuses = resolve_status_value("open");
        assert_eq!(statuses.len(), 4);
        assert!(statuses.contains(&"NEW".to_string()));
    }

    #[test]
    fn test_resolve_status_closed() {
        let statuses = resolve_status_value("closed");
        assert_eq!(statuses.len(), 7);
        assert!(statuses.contains(&"FIXED".to_string()));
        assert!(statuses.contains(&"DUPLICATE".to_string()));
    }

    #[test]
    fn test_resolve_status_specific() {
        let statuses = resolve_status_value("FIXED");
        assert_eq!(statuses, vec!["FIXED"]);
    }
}
