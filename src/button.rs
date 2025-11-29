use leptos::prelude::*;

#[component]
pub fn ActionButton(
    #[prop(optional)] icon: Option<impl IntoView>, color_variant: &'static str,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
    #[prop(optional)] class: Option<&'static str>, #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base_classes = "p-0.5 text-white border-none rounded cursor-pointer flex items-center justify-center w-5 h-5 min-w-5 transition-colors duration-200";
    let color_classes = match color_variant {
        | "primary" => "bg-[var(--color-primary,#007bff)] hover-bg-primary",
        | "secondary" => "bg-[var(--color-secondary,#6c757d)] hover-bg-secondary",
        | "danger" => "bg-[var(--color-danger,#dc3545)] hover-bg-danger",
        | _ => "bg-[var(--color-secondary,#6c757d)] hover-bg-secondary",
    };

    let button_classes = if let Some(custom_class) = class {
        format!("{} {} {}", base_classes, color_classes, custom_class)
    } else {
        format!("{} {}", base_classes, color_classes)
    };

    view! {
        <button
            class=button_classes
            on:click=on_click
        >
            {icon}
            {children.map_or_else(|| view! {}.into_any(), |children| children())}
        </button>
    }
}

#[component]
pub fn InterfaceButton(
    #[prop(optional)] icon: Option<impl IntoView>, color_variant: &'static str,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
    #[prop(optional)] class: Option<&'static str>, #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base_classes = "px-2 py-2 border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center justify-center gap-1.5 text-white min-w-[38px] min-h-[38px]";
    let color_classes = match color_variant {
        | "primary" => "bg-[var(--color-primary,#007bff)] hover-bg-primary",
        | "secondary" => "bg-[var(--color-secondary,#6c757d)] hover-bg-secondary",
        | "danger" => "bg-[var(--color-danger,#dc3545)] hover-bg-danger",
        | _ => "bg-[var(--color-secondary,#6c757d)] hover-bg-secondary",
    };

    let button_classes = if let Some(custom_class) = class {
        format!("{} {} {}", base_classes, color_classes, custom_class)
    } else {
        format!("{} {}", base_classes, color_classes)
    };

    view! {
        <button
            class=button_classes
            on:click=on_click
        >
            {icon}
            {children.map_or_else(|| view! {}.into_any(), |children| children())}
        </button>
    }
}
