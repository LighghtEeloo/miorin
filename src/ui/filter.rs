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

/// Represents a single filter option with a label and value
#[derive(Clone)]
pub struct FilterOption<T> {
    pub label: &'static str,
    pub value: T,
}

/// Generic filter button component that can be used with any filter mode type
#[component]
pub fn FilterButton<T>(
    filter_mode: RwSignal<T>,
    options: Vec<FilterOption<T>>,
) -> impl IntoView
where
    T: Clone + Copy + PartialEq + Send + Sync + 'static,
{
    let show_filter_permanent = RwSignal::new(false);
    let show_filter_hover = RwSignal::new(false);
    let options_signal = RwSignal::new(options);

    let toggle_filter = {
        let show_filter_permanent = show_filter_permanent.clone();
        move |_| {
            show_filter_permanent.update(|v| *v = !*v);
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
                    let filter_mode = filter_mode.clone();
                    let options = options_signal.get();
                    view! {
                        <div
                            class="absolute top-full right-0 mt-0.5 z-50"
                            on:mouseenter=move |_| {
                                show_filter_hover.set(true);
                            }
                        >
                            <div class="p-2 -m-2">
                                <div class="flex flex-col gap-1 bg-[var(--color-panel-bg)] border border-[var(--color-border)] rounded p-1 shadow-lg">
                                    <For
                                        each=move || {
                                            options.iter().enumerate().map(|(idx, opt)| (idx, opt.clone())).collect::<Vec<_>>()
                                        }
                                        key=|(idx, _)| *idx
                                        children=move |(_idx, option): (usize, FilterOption<T>)| {
                                            let option_value = option.value;
                                            let option_label = option.label;
                                            let filter_mode = filter_mode.clone();
                                            view! {
                                                <button
                                                    class=move || {
                                                        let base = "px-2 py-1.5 text-xs rounded cursor-pointer transition-colors text-left whitespace-nowrap";
                                                        if filter_mode.get() == option_value {
                                                            format!("{} bg-[var(--color-primary)] text-white", base)
                                                        } else {
                                                            format!("{} bg-transparent hover:bg-[var(--color-bg-secondary)] text-[var(--color-text)]", base)
                                                        }
                                                    }
                                                    on:click=move |_| {
                                                        filter_mode.set(option_value);
                                                    }
                                                >
                                                    {option_label}
                                                </button>
                                            }
                                        }
                                    />
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
pub fn GlacierFilterButton(filter_mode: RwSignal<FilterMode>) -> impl IntoView {
    let options = vec![
        FilterOption {
            label: "Show only pinned cubes",
            value: FilterMode::Pinned,
        },
        FilterOption {
            label: "Show all cubes",
            value: FilterMode::All,
        },
    ];

    view! {
        <FilterButton filter_mode=filter_mode options=options />
    }
}

#[component]
pub fn StreamFilterButton(filter_mode: RwSignal<StreamFilterMode>) -> impl IntoView {
    let options = vec![
        FilterOption {
            label: "Show only text",
            value: StreamFilterMode::Text,
        },
        FilterOption {
            label: "Show only images",
            value: StreamFilterMode::Image,
        },
        FilterOption {
            label: "Show all",
            value: StreamFilterMode::All,
        },
    ];

    view! {
        <FilterButton filter_mode=filter_mode options=options />
    }
}
