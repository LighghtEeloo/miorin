use leptos::prelude::*;

#[component]
pub fn CardList<F>(items: F) -> impl IntoView
where
    F: Fn() -> Vec<AnyView> + 'static + Send,
{
    view! {
        <div class="flex flex-col gap-2 flex-[1_1_auto] w-full max-w-full min-w-0 min-h-0 max-h-full box-border overflow-auto">
            {move || items()}
        </div>
    }
}

