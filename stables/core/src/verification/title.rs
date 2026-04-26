use super::VerificationError;

pub(super) fn normalize_verification_title(
    value: impl Into<String>,
) -> Result<String, VerificationError> {
    let title = normalize_required(value, VerificationError::MissingTitle)?;
    if title.starts_with("Verify ") {
        Ok(title)
    } else {
        Ok(format!("Verify {title}"))
    }
}

fn normalize_required(
    value: impl Into<String>,
    error: VerificationError,
) -> Result<String, VerificationError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}
