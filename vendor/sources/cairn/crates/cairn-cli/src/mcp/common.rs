use super::*;

pub(super) fn mcp_root(input: &Value) -> PathBuf {
    input["root"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(super) fn mcp_policy(input: &Value) -> PathBuf {
    input["policy"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default_policy_path()))
}

pub(super) fn mcp_required_string<'a>(
    input: &'a Value,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    input[key]
        .as_str()
        .ok_or_else(|| format!("missing string argument: {key}").into())
}

pub(super) fn attach_profile_fields(
    value: &mut Value,
    context: &crate::workflow::WorkflowProfileContext,
) {
    if context.selection.compatibility_fallback {
        if let Some(target) = value.as_object_mut() {
            target.remove("spec_effect");
        }
        return;
    }
    let Some(target) = value.as_object_mut() else {
        return;
    };
    let fields = context.receipt_fields();
    let Some(fields) = fields.as_object() else {
        return;
    };
    for (key, field) in fields {
        target.insert(key.clone(), field.clone());
    }
}

pub(super) fn reject_selector_escape(selector: &str) -> Result<(), Box<dyn Error>> {
    if selector.starts_with('/')
        || selector.contains("..")
        || selector.contains('\\')
        || selector.starts_with('.')
    {
        return Err(format!("selector escapes lifecycle bounds: {selector}").into());
    }
    Ok(())
}
