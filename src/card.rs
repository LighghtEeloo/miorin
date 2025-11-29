use leptos::prelude::*;

#[component]
pub fn CardList<F>(
    items: F,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView
where
    F: Fn() -> Vec<AnyView> + 'static + Send,
{
    let base_classes = "flex flex-col gap-2 flex-[1_1_auto] w-full max-w-full min-w-0 min-h-0 max-h-full box-border";
    let div_classes = if let Some(custom_class) = class {
        format!("{} {}", base_classes, custom_class)
    } else {
        base_classes.to_string()
    };

    view! {
        <div class=div_classes>
            {move || items()}
        </div>
    }
}

