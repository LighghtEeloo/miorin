use leptos::prelude::*;
use leptos::ev::{mousemove, mouseup};
use leptos_use::{use_document, use_event_listener};
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use std::rc::Rc;

#[component]
pub fn Panel(
    title: &'static str,
    #[prop(optional)] header_actions: Option<AnyView>,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] resizable_left: Option<bool>,
    #[prop(optional)] resizable_right: Option<bool>,
    #[prop(optional)] on_resize_left: Option<Rc<dyn Fn(f64) + 'static>>,
    #[prop(optional)] on_resize_right: Option<Rc<dyn Fn(f64) + 'static>>,
    #[prop(optional)] on_double_click_resizer: Option<Rc<dyn Fn(bool) + 'static>>,
    #[prop(optional)] is_left_panel: Option<bool>,
    children: Children,
) -> impl IntoView {
    let base_classes = "flex flex-col box-border max-h-full min-h-0 overflow-hidden h-screen relative";
    let div_classes = class.map_or_else(
        || base_classes.to_string(),
        |custom_class| format!("{} {}", base_classes, custom_class),
    );

    let show_left_resizer = resizable_left.unwrap_or(false);
    let show_right_resizer = resizable_right.unwrap_or(false);

    // State for tracking resize operations (start_x, start_width, is_left)
    let resize_state = RwSignal::new(Option::<(f64, f64, bool)>::None);
    let left_callback = on_resize_left.clone();
    let right_callback = on_resize_right.clone();
    
    // Document-level mousemove handler for resizing
    {
        let resize_state = resize_state.clone();
        let left_callback = left_callback.clone();
        let right_callback = right_callback.clone();
        let _ = use_event_listener(use_document(), mousemove, move |e: MouseEvent| {
            if let Some((start_x, start_width, is_left)) = resize_state.get() {
                let delta_x = if is_left {
                    start_x - e.client_x() as f64
                } else {
                    e.client_x() as f64 - start_x
                };
                let new_width = (start_width + delta_x).max(100.0);
                if is_left {
                    if let Some(ref callback) = left_callback {
                        callback(new_width);
                    }
                } else {
                    if let Some(ref callback) = right_callback {
                        callback(new_width);
                    }
                }
            }
        });
    }
    
    // Document-level mouseup handler to stop resizing
    {
        let resize_state = resize_state.clone();
        let _ = use_event_listener(use_document(), mouseup, move |_e: MouseEvent| {
            resize_state.set(None);
        });
    }

    // Helper function to create resizer handler
    fn create_resizer_handler(
        node_ref: NodeRef<leptos::html::Div>,
        is_left: bool,
        resize_state: RwSignal<Option<(f64, f64, bool)>>,
    ) -> impl Fn(MouseEvent) + 'static {
        move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            
            let panel_element = node_ref.get().and_then(|resizer| {
                resizer.parent_element()
            });
            
            let Some(panel_el) = panel_element.and_then(|p| p.dyn_ref::<web_sys::Element>().cloned()) else { return; };
            let start_x = e.client_x() as f64;
            // Get width using getBoundingClientRect via js_sys
            let start_width = js_sys::Reflect::get(&panel_el, &wasm_bindgen::JsValue::from_str("getBoundingClientRect"))
                .ok()
                .and_then(|f| js_sys::Function::from(f).call0(&panel_el).ok())
                .and_then(|rect| js_sys::Reflect::get(&rect, &wasm_bindgen::JsValue::from_str("width")).ok())
                .and_then(|w| w.as_f64())
                .unwrap_or(250.0);

            resize_state.set(Some((start_x, start_width, is_left)));
        }
    }

    // Left resizer handler
    let left_resizer_node_ref = NodeRef::<leptos::html::Div>::new();
    let left_resizer_on_mousedown = if show_left_resizer {
        Some(create_resizer_handler(left_resizer_node_ref.clone(), true, resize_state.clone()))
    } else {
        None
    };

    // Right resizer handler
    let right_resizer_node_ref = NodeRef::<leptos::html::Div>::new();
    let right_resizer_on_mousedown = if show_right_resizer {
        Some(create_resizer_handler(right_resizer_node_ref.clone(), false, resize_state.clone()))
    } else {
        None
    };

    view! {
        <div class=div_classes>
            {if show_left_resizer {
                let on_double_click = on_double_click_resizer.clone();
                let is_left_panel = is_left_panel.unwrap_or(false);
                view! {
                    <div
                        node_ref=left_resizer_node_ref
                        class="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize z-10 hover:bg-[var(--color-primary)] transition-colors"
                        on:mousedown=move |e| {
                            if let Some(ref handler) = left_resizer_on_mousedown {
                                handler(e);
                            }
                        }
                        on:dblclick=move |_| {
                            if let Some(ref handler) = on_double_click {
                                handler(is_left_panel);
                            }
                        }
                    />
                }.into_any()
            } else {
                view! {}.into_any()
            }}
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
            {if show_right_resizer {
                let on_double_click = on_double_click_resizer.clone();
                let is_left_panel = is_left_panel.unwrap_or(false);
                view! {
                    <div
                        node_ref=right_resizer_node_ref
                        class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize z-10 hover:bg-[var(--color-primary)] transition-colors"
                        on:mousedown=move |e| {
                            if let Some(ref handler) = right_resizer_on_mousedown {
                                handler(e);
                            }
                        }
                        on:dblclick=move |_| {
                            if let Some(ref handler) = on_double_click {
                                handler(is_left_panel);
                            }
                        }
                    />
                }.into_any()
            } else {
                view! {}.into_any()
            }}
        </div>
    }
    .into_any()
}
