use leptos::prelude::*;
use crate::color::ColorVariant;

#[component]
pub fn ActionButton(
    #[prop(optional)] icon: Option<impl IntoView>, color_variant: ColorVariant,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
    #[prop(optional)] class: Option<&'static str>, #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base_classes = "p-0.5 text-white border-none rounded cursor-pointer flex items-center justify-center w-5 h-5 min-w-5 transition-colors duration-200";
    let color_classes = color_variant.classes();

    let button_classes = class.map_or_else(
        || format!("{} {}", base_classes, color_classes),
        |custom_class| format!("{} {} {}", base_classes, color_classes, custom_class),
    );

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
    #[prop(optional)] icon: Option<impl IntoView>, color_variant: ColorVariant,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
    #[prop(optional)] class: Option<&'static str>, #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base_classes = "px-2 py-2 border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center justify-center gap-1.5 text-white min-w-[38px] min-h-[38px]";
    let color_classes = color_variant.classes();

    let button_classes = class.map_or_else(
        || format!("{} {}", base_classes, color_classes),
        |custom_class| format!("{} {} {}", base_classes, color_classes, custom_class),
    );

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
