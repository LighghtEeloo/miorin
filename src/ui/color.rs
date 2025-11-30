pub enum ColorVariant {
    Primary,
    Secondary,
    Danger,
}

impl ColorVariant {
    pub fn classes(&self) -> &'static str {
        match self {
            | ColorVariant::Primary => "bg-[var(--color-primary,#007bff)] hover-bg-primary",
            | ColorVariant::Secondary => "bg-[var(--color-secondary,#6c757d)] hover-bg-secondary",
            | ColorVariant::Danger => "bg-[var(--color-danger,#dc3545)] hover-bg-danger",
        }
    }
}

impl From<&'static str> for ColorVariant {
    fn from(s: &'static str) -> Self {
        match s {
            | "primary" => ColorVariant::Primary,
            | "secondary" => ColorVariant::Secondary,
            | "danger" => ColorVariant::Danger,
            | _ => ColorVariant::Secondary,
        }
    }
}
