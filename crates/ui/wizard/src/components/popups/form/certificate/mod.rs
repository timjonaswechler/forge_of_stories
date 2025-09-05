use crate::components::popups::form::{FormField, FormFieldKind, FormPopup, FormSchema, FormState};

/// Build a certificate wizard popup (FormPopup) with common fields:
/// - Common Name (CN) [Text, required]
/// - Organization (O) [Text, optional]
/// - Country (C) [Text, 2-letter ISO code, required]
/// - Subject Alternative Names (SANs) [List of strings; DNS/IP; item validator: non-empty, no whitespace]
/// - Validity (days) [Number, required, 1..=825]
/// - Algorithm [Select: RSA, ECDSA]
/// - Output Path [Path, required]
/// - Self-signed [Bool]
///
/// Returns a configured FormPopup with basic validators and some sensible defaults for:
/// - validity_days = 365
/// - algorithm = "RSA"
/// - self_signed = true
pub fn certificate_wizard_popup() -> FormPopup {
    // Reusable validators
    let not_empty = |label: &'static str| {
        move |s: &str| {
            if s.trim().is_empty() {
                Err(format!("{label} must not be empty"))
            } else {
                Ok(())
            }
        }
    };

    let country_code = |s: &str| {
        let cc = s.trim();
        if cc.len() != 2 || !cc.chars().all(|c| c.is_ascii_alphabetic()) {
            Err("Country must be a 2-letter ISO code (A-Z)".to_string())
        } else {
            Ok(())
        }
    };

    let number_in_range = |min: i64, max: i64| {
        move |s: &str| match s.trim().parse::<i64>() {
            Ok(n) if n >= min && n <= max => Ok(()),
            _ => Err(format!("Must be a number in range [{min}..{max}]")),
        }
    };

    let san_item = |s: &str| {
        let v = s.trim();
        if v.is_empty() {
            return Err("SAN item must not be empty".to_string());
        }
        if v.chars().any(|c| c.is_whitespace()) {
            return Err("SAN item must not contain whitespace".to_string());
        }
        Ok(())
    };

    // Schema (layout/fields)
    let schema = FormSchema::new(
        "Certificate Setup",
        vec![
            // CN (Common Name)
            FormField::new("cn", "Common Name (CN)", FormFieldKind::Text)
                .help("Primary certificate subject (e.g., example.com)")
                .validator(not_empty("Common Name")),

            // Organization (optional)
            FormField::new("org", "Organization (O)", FormFieldKind::Text)
                .help("Optional organization name"),

            // Country (2-letter ISO)
            FormField::new("country", "Country (C)", FormFieldKind::Text)
                .help("2-letter ISO code (e.g., US, DE)")
                .validator(country_code),

            // SANs list
            FormField::new("sans", "Subject Alt Names (SANs)", FormFieldKind::ListString)
                .help("Add DNS names or IP addresses (Insert to add an item)")
                .validator(san_item),

            // Validity (days)
            FormField::new("validity_days", "Validity (days)", FormFieldKind::Number)
                .help("Number of days the certificate remains valid (1..=825)")
                .validator(number_in_range(1, 825)),

            // Algorithm
            FormField::new(
                "algorithm",
                "Key Algorithm",
                FormFieldKind::Select {
                    options: vec!["RSA".to_string(), "ECDSA".to_string()],
                },
            )
            .help("Choose the key algorithm"),

            // Output path
            FormField::new("output_path", "Output Path", FormFieldKind::Path)
                .help("Directory or file path to write the certificate (and key)")
                .validator(not_empty("Output Path")),

            // Self-signed
            FormField::new("self_signed", "Self-signed", FormFieldKind::Bool)
                .help("Enable to generate a self-signed certificate"),
        ],
    )
    .description("Provide the details required to generate a certificate. \
                  Use Up/Down to navigate, Enter to edit a field, Insert to add SAN entries, and Enter to submit.")
    .min_size(64, 80);

    // Initial state with sensible defaults
    let mut state = FormState::default();
    state.set_value("validity_days", "365");
    state.set_value("algorithm", "RSA");
    state.set_value("self_signed", "true");
    // Optional: pre-fill country or other defaults if you like
    // state.set_value("country", "US");

    FormPopup::new(schema).with_state(state)
}
