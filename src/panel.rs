use leptos::prelude::*;

#[component]
pub fn Panel(
    title: &'static str, #[prop(optional)] header_actions: Option<AnyView>, children: Children,
) -> impl IntoView {
    let has_actions = header_actions.is_some();

    view! {
        <div class="flex flex-col box-border max-h-full min-h-0 overflow-hidden h-screen">
            <div 
                class="flex flex-col h-full max-h-full min-h-0 border-r overflow-hidden border-[var(--color-border)] bg-[var(--color-panel-bg)]"
            >
                <div 
                    class=if has_actions {
                        "shrink-0 p-4 border-b flex justify-between items-center border-[var(--color-border)] bg-[var(--color-panel-header-bg)]"
                    } else {
                        "shrink-0 p-4 border-b border-[var(--color-border)] bg-[var(--color-panel-header-bg)]"
                    }
                >
                    <h2 class="m-0 text-base font-semibold text-[var(--color-text)]">
                        {title}
                    </h2>
                    {header_actions}
                </div>
                <div class="flex flex-col flex-1 min-h-0 overflow-y-auto overflow-x-hidden p-2 box-border">
                    {children()}
                </div>
            </div>
        </div>
    }
    .into_any()
}
