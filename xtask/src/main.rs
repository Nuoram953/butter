use butter::rules::{file::FileRuleConfig, file_name::FileNameRuleConfig};
use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::Value;
use std::fs;

fn resolve_type(prop: &Value, root: &Value) -> String {
    if let Some(reference) = prop.get("$ref").and_then(|r| r.as_str()) {
        if let Some(name) = reference.rsplit('/').next() {
            if let Some(resolved) = root.get("$defs").and_then(|d| d.get(name)) {
                return resolve_type(resolved, root);
            }
        }
        return "object".to_string();
    }

    let ty = prop
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("object");

    // If it's an enum, show the allowed values instead of just "string".
    if let Some(values) = prop.get("enum").and_then(|e| e.as_array()) {
        let joined = values
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return format!("{ty} (`{joined}`)");
    }

    ty.to_string()
}

fn schema_to_markdown(schema: &Value, title: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("### `{title}`\n\n"));
    if let Some(desc) = schema.get("description").and_then(|d| d.as_str()) {
        out.push_str(desc);
        out.push_str("\n\n");
    }
    out.push_str("| Field | Type | Required | Description |\n");
    out.push_str("|---|---|---|---|\n");
    let required: Vec<&str> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (name, prop) in props {
            let ty = resolve_type(prop, schema);
            let desc = prop
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let is_required = required.contains(&name.as_str());
            out.push_str(&format!(
                "| `{name}` | {ty} | {} | {desc} |\n",
                if is_required { "yes" } else { "no" }
            ));
        }
    }
    out.push('\n');
    out
}
fn update_readme_section(readme_path: &str, key: &str, content: &str) -> std::io::Result<()> {
    let original = fs::read_to_string(readme_path)?;
    let start_marker = format!("<!-- SCHEMA:{key}:START -->");
    let end_marker = format!("<!-- SCHEMA:{key}:END -->");

    let start = original
        .find(&start_marker)
        .expect("start marker not found in README");
    let end = original
        .find(&end_marker)
        .expect("end marker not found in README");

    let before = &original[..start + start_marker.len()];
    let after = &original[end..];

    let updated = format!("{before}\n{content}\n{after}");
    fs::write(readme_path, updated)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let schema = schema_for!(FileRuleConfig);
    let value: Value = serde_json::to_value(&schema).unwrap();
    let md = schema_to_markdown(&value, "file");
    update_readme_section("README.md", "file", &md)?;

    let schema = schema_for!(FileNameRuleConfig);

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    let value: Value = serde_json::to_value(&schema).unwrap();
    let md = schema_to_markdown(&value, "file_name");
    update_readme_section("README.md", "file_name", &md)?;
    Ok(())
}
