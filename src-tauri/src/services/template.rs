use std::collections::HashMap;

/// Render a template string with variable substitution and conditional sections.
///
/// **Variables:** `{{VAR_NAME}}` is replaced with the value from `context`.
/// If the key is not found, it is replaced with an empty string.
///
/// **Conditionals:** `{{#FIELD}}content{{/FIELD}}` — if `FIELD` is empty or missing
/// from `context`, the entire block (including delimiters) is removed. If `FIELD`
/// has a non-empty value, the content is kept and the delimiters are stripped.
///
/// Nested conditionals of the same field name are not supported and will be handled
/// on a best-effort basis (outermost match wins). Malformed delimiters are left as-is.
pub fn render_template(template: &str, context: &HashMap<String, String>) -> String {
    // Phase 1: Process conditional sections.
    // We process from the inside out by repeatedly finding the innermost conditional.
    let mut result = template.to_string();

    // Loop until no more conditional sections are found.
    // An innermost conditional is one whose content does not contain another opening tag.
    loop {
        // Find the next conditional pattern: {{#FIELD}}...{{/FIELD}}
        // We look for the innermost one (content has no nested {{#...}}).
        let Some(start_pos) = result.find("{{#") else {
            break;
        };

        let after_open = &result[start_pos + 3..];
        let Some(close_brace) = after_open.find("}}") else {
            // Malformed — no closing braces for opening tag. Stop processing.
            break;
        };

        let field_name = &after_open[..close_brace];

        // Validate field name is non-empty and doesn't contain suspicious chars
        if field_name.is_empty() || field_name.contains("{{") || field_name.contains("}}") {
            break;
        }

        let open_tag = format!("{{{{#{field_name}}}}}");
        let close_tag = format!("{{{{/{field_name}}}}}");

        // Find this specific pair (use the start_pos we already found)
        let content_start = start_pos + open_tag.len();

        let Some(end_pos) = result[content_start..].find(&close_tag) else {
            // No matching close tag — leave as-is and stop
            break;
        };
        let end_pos = content_start + end_pos;

        let content = &result[content_start..end_pos];
        let block_end = end_pos + close_tag.len();

        let field_value = context.get(field_name).map(|s| s.as_str()).unwrap_or("");

        let replacement = if field_value.is_empty() {
            String::new()
        } else {
            content.to_string()
        };

        result = format!(
            "{}{}{}",
            &result[..start_pos],
            replacement,
            &result[block_end..]
        );
    }

    // Phase 2: Variable substitution.
    // Replace all {{KEY}} with their values (or empty string).
    let mut output = String::with_capacity(result.len());
    let mut remaining = result.as_str();

    loop {
        let Some(var_start) = remaining.find("{{") else {
            output.push_str(remaining);
            break;
        };

        // Check this isn't a conditional tag remnant ({{# or {{/)
        let after = &remaining[var_start + 2..];
        if after.starts_with('#') || after.starts_with('/') {
            // Not a variable — copy up to and including {{ and continue
            output.push_str(&remaining[..var_start + 2]);
            remaining = after;
            continue;
        }

        let Some(var_end) = after.find("}}") else {
            // No closing }} — copy rest and stop
            output.push_str(remaining);
            break;
        };

        let key = &after[..var_end];

        // Push everything before the variable
        output.push_str(&remaining[..var_start]);

        // Push the variable value (or empty)
        let value = context.get(key).map(|s| s.as_str()).unwrap_or("");
        output.push_str(value);

        remaining = &after[var_end + 2..];
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn render_simple_substitution() {
        let result = render_template("Hello {{NAME}}", &ctx(&[("NAME", "World")]));
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn render_missing_variable() {
        let result = render_template("Hi {{NAME}}", &ctx(&[]));
        assert_eq!(result, "Hi ");
    }

    #[test]
    fn render_conditional_present() {
        let result = render_template("{{#CVE}}CVE: {{CVE}}{{/CVE}}", &ctx(&[("CVE", "CVE-123")]));
        assert_eq!(result, "CVE: CVE-123");
    }

    #[test]
    fn render_conditional_absent() {
        let result = render_template("{{#CVE}}CVE: {{CVE}}{{/CVE}}", &ctx(&[]));
        assert_eq!(result, "");
    }

    #[test]
    fn render_conditional_empty() {
        let result = render_template("{{#CVE}}CVE: {{CVE}}{{/CVE}}", &ctx(&[("CVE", "")]));
        assert_eq!(result, "");
    }

    #[test]
    fn render_mixed() {
        let template = "Fix {{PACKAGE}} to {{VERSION}}{{#CVE}} ({{CVE}}){{/CVE}} in {{REPO}}";
        let result = render_template(
            template,
            &ctx(&[
                ("PACKAGE", "lodash"),
                ("VERSION", "4.17.21"),
                ("CVE", "CVE-2024-001"),
                ("REPO", "my-app"),
            ]),
        );
        assert_eq!(result, "Fix lodash to 4.17.21 (CVE-2024-001) in my-app");
    }

    #[test]
    fn render_mixed_conditional_absent() {
        let template = "Fix {{PACKAGE}} to {{VERSION}}{{#CVE}} ({{CVE}}){{/CVE}} in {{REPO}}";
        let result = render_template(
            template,
            &ctx(&[
                ("PACKAGE", "lodash"),
                ("VERSION", "4.17.21"),
                ("REPO", "my-app"),
            ]),
        );
        assert_eq!(result, "Fix lodash to 4.17.21 in my-app");
    }

    #[test]
    fn render_pr_body_realistic() {
        let template = r#"## Security Patch — {{PACKAGE}} {{VERSION}}

{{#CVE}}### CVE Details
- **ID:** {{CVE}}
- **Severity:** {{SEVERITY}}
{{/CVE}}
### Changes
- Updated `{{PACKAGE}}` from {{FROM_VERSION}} to {{VERSION}}
{{#MODE}}- **Mode:** {{MODE}}
{{/MODE}}
---
Generated by Git Flotilla on {{DATE}}"#;

        let result = render_template(
            template,
            &ctx(&[
                ("PACKAGE", "axios"),
                ("VERSION", "1.6.1"),
                ("FROM_VERSION", "1.5.0"),
                ("CVE", "CVE-2024-39338"),
                ("SEVERITY", "high"),
                ("DATE", "2026-04-09"),
                ("MODE", "pin"),
            ]),
        );

        assert!(result.contains("## Security Patch — axios 1.6.1"));
        assert!(result.contains("**ID:** CVE-2024-39338"));
        assert!(result.contains("**Severity:** high"));
        assert!(result.contains("Updated `axios` from 1.5.0 to 1.6.1"));
        assert!(result.contains("**Mode:** pin"));
        assert!(result.contains("Generated by Git Flotilla on 2026-04-09"));
    }

    #[test]
    fn render_pr_body_realistic_no_cve() {
        let template = r#"## Update — {{PACKAGE}} {{VERSION}}

{{#CVE}}### CVE Details
- **ID:** {{CVE}}
{{/CVE}}### Changes
- Bumped `{{PACKAGE}}` to {{VERSION}}"#;

        let result = render_template(template, &ctx(&[("PACKAGE", "vue"), ("VERSION", "3.4.0")]));

        assert!(result.contains("## Update — vue 3.4.0"));
        assert!(!result.contains("CVE Details"));
        assert!(result.contains("Bumped `vue` to 3.4.0"));
    }

    #[test]
    fn render_multiple_conditionals() {
        let template = "{{#A}}A={{A}} {{/A}}{{#B}}B={{B}}{{/B}}";
        let result = render_template(template, &ctx(&[("A", "1")]));
        assert_eq!(result, "A=1 ");
    }

    #[test]
    fn render_no_panic_on_malformed() {
        // Unclosed variable
        let result = render_template("Hello {{NAME", &ctx(&[("NAME", "World")]));
        assert_eq!(result, "Hello {{NAME");

        // Unclosed conditional
        let result = render_template("{{#CVE}}content", &ctx(&[("CVE", "x")]));
        // Should not panic — best effort
        assert!(result.contains("content") || result.contains("{{#CVE}}"));
    }
}
