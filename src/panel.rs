use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::MouseEvent;
use std::rc::Rc;
use std::cell::RefCell;

#[component]
pub fn Panel(
    title: &'static str,
    #[prop(optional)] header_actions: Option<AnyView>,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] resizable_left: Option<bool>,
    #[prop(optional)] resizable_right: Option<bool>,
    #[prop(optional)] on_resize_left: Option<Rc<dyn Fn(f64) + 'static>>,
    #[prop(optional)] on_resize_right: Option<Rc<dyn Fn(f64) + 'static>>,
    children: Children,
) -> impl IntoView {
    let base_classes = "flex flex-col box-border max-h-full min-h-0 overflow-hidden h-screen relative";
    let div_classes = if let Some(custom_class) = class {
        format!("{} {}", base_classes, custom_class)
    } else {
        base_classes.to_string()
    };

    let show_left_resizer = resizable_left.unwrap_or(false);
    let show_right_resizer = resizable_right.unwrap_or(false);

    // Helper function to create resizer handler
    fn create_resizer_handler(
        on_resize: Option<Rc<dyn Fn(f64)>>,
        node_ref: NodeRef<leptos::html::Div>,
        is_left: bool,
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

            let on_resize_opt = on_resize.as_ref().map(|cb| cb.clone());
            let on_mousemove = Closure::wrap(Box::new({
                let start_x = start_x;
                let start_width = start_width;
                let on_resize_opt = on_resize_opt.clone();
                move |e: MouseEvent| {
                    let delta_x = if is_left {
                        start_x - e.client_x() as f64
                    } else {
                        e.client_x() as f64 - start_x
                    };
                    let new_width = (start_width + delta_x).max(100.0);
                    if let Some(ref callback) = on_resize_opt {
                        callback(new_width);
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let cleanup_state = Rc::new(RefCell::new(Some(on_mousemove)));
            let cleanup_state_clone = cleanup_state.clone();
            
            let on_mouseup = Closure::wrap(Box::new(move |_e: MouseEvent| {
                let document = web_sys::window()
                    .and_then(|w| w.document())
                    .expect("should have document");
                let mut state = cleanup_state_clone.borrow_mut();
                if let Some(ref on_mousemove) = *state {
                    document.remove_event_listener_with_callback("mousemove", on_mousemove.as_ref().unchecked_ref()).ok();
                }
                *state = None;
            }) as Box<dyn FnMut(_)>);

            let document = web_sys::window()
                .and_then(|w| w.document())
                .expect("should have document");
            if let Some(ref on_mousemove) = *cleanup_state.borrow() {
                document.add_event_listener_with_callback("mousemove", on_mousemove.as_ref().unchecked_ref()).ok();
            }
            document.add_event_listener_with_callback("mouseup", on_mouseup.as_ref().unchecked_ref()).ok();
            on_mouseup.forget();
        }
    }

    // Left resizer handler
    let left_resizer_node_ref = NodeRef::<leptos::html::Div>::new();
    let left_resizer_on_mousedown = if show_left_resizer {
        Some(create_resizer_handler(on_resize_left.clone(), left_resizer_node_ref.clone(), true))
    } else {
        None
    };

    // Right resizer handler
    let right_resizer_node_ref = NodeRef::<leptos::html::Div>::new();
    let right_resizer_on_mousedown = if show_right_resizer {
        Some(create_resizer_handler(on_resize_right.clone(), right_resizer_node_ref.clone(), false))
    } else {
        None
    };

    view! {
        <div class=div_classes>
            {if show_left_resizer {
                view! {
                    <div
                        node_ref=left_resizer_node_ref
                        class="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize z-10 hover:bg-[var(--color-primary)] transition-colors"
                        on:mousedown=move |e| {
                            if let Some(ref handler) = left_resizer_on_mousedown {
                                handler(e);
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
                view! {
                    <div
                        node_ref=right_resizer_node_ref
                        class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize z-10 hover:bg-[var(--color-primary)] transition-colors"
                        on:mousedown=move |e| {
                            if let Some(ref handler) = right_resizer_on_mousedown {
                                handler(e);
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
