use leptos::prelude::*;

#[component]
pub fn Toggle(enabled: RwSignal<bool>, on_change: impl Fn(bool) + 'static) -> impl IntoView {
    view! {
        <label class="relative inline-block w-9 h-5">
            <input
                type="checkbox"
                class="opacity-0 w-0 h-0"
                checked=enabled
                on:change=move |_| {
                    let new_value = !enabled.get();
                    enabled.set(new_value);
                    on_change(new_value);
                }
            />
            <span
                class="absolute cursor-pointer top-0 left-0 right-0 bottom-0 rounded-full transition-all duration-300"
                style=move || if enabled.get() { "background-color: var(--color-primary);" } else { "background-color: var(--color-toggle-bg);" }
            >
                <span
                    class="absolute h-[14px] w-[14px] left-[3px] bottom-[3px] rounded-full transition-all duration-300"
                    style=move || {
                        let thumb_color = "background-color: var(--color-toggle-thumb);";
                        let transform = if enabled.get() { "transform: translateX(16px);" } else { "transform: translateX(0);" };
                        format!("{} {}", thumb_color, transform)
                    }
                ></span>
            </span>
        </label>
    }
}
