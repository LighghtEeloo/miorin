use crate::ui::{button, color::ColorVariant};
use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::LuSlidersHorizontal;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FilterMode {
    Pinned,
    All,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StreamFilterMode {
    Text,
    Image,
    All,
}

#[component]
pub fn FilterButton(
    filter_mode: RwSignal<FilterMode>,
) -> impl IntoView {
    let show_filter_permanent = RwSignal::new(false);
    let show_filter_hover = RwSignal::new(false);

    let toggle_filter = {
        let show_filter_permanent = show_filter_permanent.clone();
        move |_| {
            show_filter_permanent.update(|v| *v = !*v);
        }
    };

    let set_filter_pinned = {
        let filter_mode = filter_mode.clone();
        move |_| {
            filter_mode.set(FilterMode::Pinned);
        }
    };

    let set_filter_all = {
        let filter_mode = filter_mode.clone();
        move |_| {
            filter_mode.set(FilterMode::All);
        }
    };

    view! {
        <div
            class="relative"
            on:mouseenter=move |_| {
                show_filter_hover.set(true);
            }
            on:mouseleave=move |_| {
                if !show_filter_permanent.get() {
                    show_filter_hover.set(false);
                }
            }
        >
            <button::ActionButton
                icon=view! { <Icon icon=LuSlidersHorizontal width="12" height="12" /> }
                color_variant=ColorVariant::Secondary
                on_click=toggle_filter
            />
            {move || {
                let show = show_filter_permanent.get() || show_filter_hover.get();
                if show {
                    view! {
                        <div 
                            class="absolute top-full right-0 mt-0.5 z-50"
                            on:mouseenter=move |_| {
                                show_filter_hover.set(true);
                            }
                        >
                            <div class="p-2 -m-2">
                                <div class="flex flex-col gap-1 bg-[var(--color-panel-bg)] border border-[var(--color-border)] rounded p-1 shadow-lg">
                                    <button
                                        class=move || {
                                            let base = "px-2 py-1.5 text-xs rounded cursor-pointer transition-colors text-left whitespace-nowrap";
                                            if filter_mode.get() == FilterMode::Pinned {
                                                format!("{} bg-[var(--color-primary)] text-white", base)
                                            } else {
                                                format!("{} bg-transparent hover:bg-[var(--color-bg-secondary)] text-[var(--color-text)]", base)
                                            }
                                        }
                                        on:click=set_filter_pinned
                                    >
                                        "Show only pinned cubes"
                                    </button>
                                    <button
                                        class=move || {
                                            let base = "px-2 py-1.5 text-xs rounded cursor-pointer transition-colors text-left whitespace-nowrap";
                                            if filter_mode.get() == FilterMode::All {
                                                format!("{} bg-[var(--color-primary)] text-white", base)
                                            } else {
                                                format!("{} bg-transparent hover:bg-[var(--color-bg-secondary)] text-[var(--color-text)]", base)
                                            }
                                        }
                                        on:click=set_filter_all
                                    >
                                        "Show all cubes"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn StreamFilterButton(
    filter_mode: RwSignal<StreamFilterMode>,
) -> impl IntoView {
    let show_filter_permanent = RwSignal::new(false);
    let show_filter_hover = RwSignal::new(false);

    let toggle_filter = {
        let show_filter_permanent = show_filter_permanent.clone();
        move |_| {
            show_filter_permanent.update(|v| *v = !*v);
        }
    };

    let set_filter_text = {
        let filter_mode = filter_mode.clone();
        move |_| {
            filter_mode.set(StreamFilterMode::Text);
        }
    };

    let set_filter_image = {
        let filter_mode = filter_mode.clone();
        move |_| {
            filter_mode.set(StreamFilterMode::Image);
        }
    };

    let set_filter_all = {
        let filter_mode = filter_mode.clone();
        move |_| {
            filter_mode.set(StreamFilterMode::All);
        }
    };

    view! {
        <div
            class="relative"
            on:mouseenter=move |_| {
                show_filter_hover.set(true);
            }
            on:mouseleave=move |_| {
                if !show_filter_permanent.get() {
                    show_filter_hover.set(false);
                }
            }
        >
            <button::ActionButton
                icon=view! { <Icon icon=LuSlidersHorizontal width="12" height="12" /> }
                color_variant=ColorVariant::Secondary
                on_click=toggle_filter
            />
            {move || {
                let show = show_filter_permanent.get() || show_filter_hover.get();
                if show {
                    view! {
                        <div 
                            class="absolute top-full right-0 mt-0.5 z-50"
                            on:mouseenter=move |_| {
                                show_filter_hover.set(true);
                            }
                        >
                            <div class="p-2 -m-2">
                                <div class="flex flex-col gap-1 bg-[var(--color-panel-bg)] border border-[var(--color-border)] rounded p-1 shadow-lg">
                                    <button
                                        class=move || {
                                            let base = "px-2 py-1.5 text-xs rounded cursor-pointer transition-colors text-left whitespace-nowrap";
                                            if filter_mode.get() == StreamFilterMode::Text {
                                                format!("{} bg-[var(--color-primary)] text-white", base)
                                            } else {
                                                format!("{} bg-transparent hover:bg-[var(--color-bg-secondary)] text-[var(--color-text)]", base)
                                            }
                                        }
                                        on:click=set_filter_text
                                    >
                                        "Show only text"
                                    </button>
                                    <button
                                        class=move || {
                                            let base = "px-2 py-1.5 text-xs rounded cursor-pointer transition-colors text-left whitespace-nowrap";
                                            if filter_mode.get() == StreamFilterMode::Image {
                                                format!("{} bg-[var(--color-primary)] text-white", base)
                                            } else {
                                                format!("{} bg-transparent hover:bg-[var(--color-bg-secondary)] text-[var(--color-text)]", base)
                                            }
                                        }
                                        on:click=set_filter_image
                                    >
                                        "Show only images"
                                    </button>
                                    <button
                                        class=move || {
                                            let base = "px-2 py-1.5 text-xs rounded cursor-pointer transition-colors text-left whitespace-nowrap";
                                            if filter_mode.get() == StreamFilterMode::All {
                                                format!("{} bg-[var(--color-primary)] text-white", base)
                                            } else {
                                                format!("{} bg-transparent hover:bg-[var(--color-bg-secondary)] text-[var(--color-text)]", base)
                                            }
                                        }
                                        on:click=set_filter_all
                                    >
                                        "Show all"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}

