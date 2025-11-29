use leptos::prelude::*;

#[component]
pub fn Panel(
    title: &'static str, #[prop(optional)] header_actions: Option<AnyView>, children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-col box-border max-h-full min-h-0 overflow-hidden h-screen">
            <div 
                class="flex flex-col h-full max-h-full min-h-0 border-r overflow-hidden border-[var(--color-border)] bg-[var(--color-panel-bg)]"
            >
                <div 
                    class="shrink-0 py-2 pl-3 pr-2 border-b flex justify-between items-center border-[var(--color-border)] bg-[var(--color-panel-header-bg)]"
                >
                    <h2 class="m-0 text-sm font-medium text-[var(--color-text)]">
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
