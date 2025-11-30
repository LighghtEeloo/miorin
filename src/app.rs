use crate::ui::{
    button, card,
    color::ColorVariant,
    filter::{FilterButton, FilterMode},
    panel,
};
use crate::tauri_api;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_use::{use_debounce_fn_with_arg, use_element_hover};
use icondata::{LuPlus, LuSettings, LuTrash2};
use miorin_core::prelude::*;
use wasm_bindgen_futures::spawn_local;
use base64::{Engine as _, engine::general_purpose};

#[component]
pub fn App() -> impl IntoView {
    // Load raw entries from database
    let raw_entries = RwSignal::new(Vec::<Raw>::new());
    let raw_entries_for_load = raw_entries.clone();
    spawn_local(async move {
        web_sys::console::log_1(&"Starting to load raw entries from database...".into());
        match tauri_api::get_all_raw_entries().await {
            | Ok(entries) => {
                web_sys::console::log_1(
                    &format!("Successfully loaded {} raw entries", entries.len()).into(),
                );
                raw_entries_for_load.set(entries);
            }
            | Err(e) => {
                web_sys::console::error_1(&format!("Failed to load raw entries: {}", e).into());
            }
        }
        web_sys::console::log_1(&"Finished loading raw entries".into());
    });

    // Listen for raw-entry-created events to refresh the stream panel
    let raw_entries_for_event = raw_entries.clone();
    spawn_local(async move {
        if let Err(e) = tauri_api::listen_to_event("raw-entry-created", move |_event| {
            web_sys::console::log_1(&"Received raw-entry-created event".into());
            let raw_entries = raw_entries_for_event.clone();
            spawn_local(async move {
                web_sys::console::log_1(&"Refreshing raw entries after new entry created...".into());
                match tauri_api::get_all_raw_entries().await {
                    | Ok(entries) => {
                        web_sys::console::log_1(
                            &format!("Refreshed raw entries, now have {} entries", entries.len()).into(),
                        );
                        raw_entries.set(entries);
                    }
                    | Err(e) => {
                        web_sys::console::error_1(&format!("Failed to refresh raw entries: {}", e).into());
                    }
                }
            });
        }) {
            web_sys::console::error_1(&format!("Failed to listen to raw-entry-created event: {}", e).into());
        } else {
            web_sys::console::log_1(&"Successfully set up listener for raw-entry-created event".into());
        }
    });

    // Load cubes from database
    let cubes = RwSignal::new(Vec::<Cube>::new());
    spawn_local(async move {
        match tauri_api::get_all_cubes().await {
            | Ok(cubes_data) => cubes.set(cubes_data),
            | Err(e) => {
                web_sys::console::error_1(&format!("Failed to load cubes: {}", e).into());
            }
        }
    });

    // Settings visibility state
    let show_settings = RwSignal::new(false);

    // Panel widths - load once at app level so they persist across view switches
    let left_width = RwSignal::new(250.0);
    let right_width = RwSignal::new(300.0);
    let left_width = left_width.clone();
    let right_width = right_width.clone();
    spawn_local(async move {
        match tauri_api::get_panel_left_width().await {
            | Ok(width) => left_width.set(width),
            | Err(e) => {
                web_sys::console::warn_1(&format!("Failed to load left panel width: {}", e).into());
            }
        }
        match tauri_api::get_panel_right_width().await {
            | Ok(width) => right_width.set(width),
            | Err(e) => {
                web_sys::console::warn_1(
                    &format!("Failed to load right panel width: {}", e).into(),
                );
            }
        }
    });

    // Settings button handler
    let open_settings = {
        let show_settings = show_settings.clone();
        move |_| {
            show_settings.set(true);
        }
    };

    let close_settings = {
        let show_settings = show_settings.clone();
        move || {
            show_settings.set(false);
        }
    };

    view! {
        {move || {
            if show_settings.get() {
                view! {
                    <crate::settings::Settings
                        close_settings=close_settings.clone()
                        raw_entries_signal=raw_entries.clone()
                    />
                }.into_any()
            } else {
                view! {
                    <MainView
                        cubes=cubes
                        raw_entries=raw_entries
                        open_settings=open_settings
                        left_width=left_width
                        right_width=right_width
                    />
                }.into_any()
            }
        }}
    }
}

#[component]
fn MainView(
    cubes: RwSignal<Vec<Cube>>, raw_entries: RwSignal<Vec<Raw>>,
    open_settings: impl Fn(web_sys::MouseEvent) + 'static, left_width: RwSignal<f64>,
    right_width: RwSignal<f64>,
) -> impl IntoView {
    // Debounced save function for left panel width
    let save_left_width = move |width: f64| {
        spawn_local(async move {
            if let Err(e) = tauri_api::set_panel_left_width(width).await {
                web_sys::console::error_1(
                    &format!("Failed to save left panel width: {}", e).into(),
                );
            }
        });
    };
    let debounced_save_left = use_debounce_fn_with_arg(save_left_width, 500.0);

    // Debounced save function for right panel width
    let save_right_width = move |width: f64| {
        spawn_local(async move {
            if let Err(e) = tauri_api::set_panel_right_width(width).await {
                web_sys::console::error_1(
                    &format!("Failed to save right panel width: {}", e).into(),
                );
            }
        });
    };
    let debounced_save_right = use_debounce_fn_with_arg(save_right_width, 500.0);

    let on_resize_left = {
        let left_width = left_width.clone();
        let debounced_save_left = debounced_save_left.clone();
        std::rc::Rc::new(move |new_width: f64| {
            left_width.set(new_width);
            debounced_save_left(new_width);
        }) as std::rc::Rc<dyn Fn(f64)>
    };

    let on_resize_right = {
        let right_width = right_width.clone();
        let debounced_save_right = debounced_save_right.clone();
        std::rc::Rc::new(move |new_width: f64| {
            right_width.set(new_width);
            debounced_save_right(new_width);
        }) as std::rc::Rc<dyn Fn(f64)>
    };

    // Double-click handler to shrink narrower panel when panels meet
    let on_double_click_resizer = {
        let left_width = left_width.clone();
        let right_width = right_width.clone();
        let debounced_save_left = debounced_save_left.clone();
        let debounced_save_right = debounced_save_right.clone();
        std::rc::Rc::new(move |_is_left_panel: bool| {
            // Get viewport width
            let viewport_width = web_sys::window()
                .and_then(|w| w.inner_width().ok())
                .and_then(|w| w.as_f64())
                .unwrap_or(1920.0);
            
            let current_left = left_width.get();
            let current_right = right_width.get();
            let total_panel_width = current_left + current_right;
            
            // Check if panels are meeting (within 10px threshold)
            let threshold = 10.0;
            if total_panel_width >= viewport_width - threshold {
                // Find the wider panel
                let (wider_width, is_left_wider) = if current_left >= current_right {
                    (current_left, true)
                } else {
                    (current_right, false)
                };
                
                // Shrink the wider panel by 30% or minimum 100px, whichever is larger
                let shrink_amount = (wider_width * 0.3).max(100.0);
                let new_width = (wider_width - shrink_amount).max(100.0);
                
                if is_left_wider {
                    left_width.set(new_width);
                    debounced_save_left(new_width);
                } else {
                    right_width.set(new_width);
                    debounced_save_right(new_width);
                }
            }
        }) as std::rc::Rc<dyn Fn(bool)>
    };

    view! {
        <div
            class="grid h-screen max-h-screen overflow-hidden bg-[var(--color-bg)]"
            style=move || format!("grid-template-columns: {}px 1fr {}px;", left_width.get(), right_width.get())
        >
            <GlacierPanel
                cubes=cubes
                on_resize_right=on_resize_left.clone()
                on_double_click_resizer=on_double_click_resizer.clone()
                is_left_panel=true
            />
            <Workspace />
            <StreamPanel
                raw_entries=raw_entries
                open_settings=open_settings
                on_resize_left=on_resize_right.clone()
                on_double_click_resizer=on_double_click_resizer.clone()
                is_left_panel=false
            />
        </div>
    }
}

#[component]
fn GlacierPanel(
    cubes: RwSignal<Vec<Cube>>, 
    on_resize_right: std::rc::Rc<dyn Fn(f64) + 'static>,
    on_double_click_resizer: std::rc::Rc<dyn Fn(bool) + 'static>,
    is_left_panel: bool,
) -> impl IntoView {
    let create_cube_action = move |_| {
        let cubes_signal = cubes.clone();
        spawn_local(async move {
            // Create a dummy point cube with paragraph text
            let dummy_point = Point::Text(Text {
                style: TextStyle::Paragraph,
                text: RichText {
                    segments: vec![RichTextSegment {
                        text: "Text".to_string(),
                        marks: TextMarks::default(),
                    }],
                },
            });

            web_sys::console::log_1(&"Creating cube...".into());
            match tauri_api::create_cube(true, CubeContent::Point(dummy_point)).await {
                | Ok(cube) => {
                    web_sys::console::log_1(
                        &format!("Cube created successfully with id: {}", cube.id.0).into(),
                    );
                    // Reload cubes
                    match tauri_api::get_all_cubes().await {
                        | Ok(cubes_data) => {
                            web_sys::console::log_1(
                                &format!("Reloaded {} cubes", cubes_data.len()).into(),
                            );
                            cubes_signal.set(cubes_data);
                        }
                        | Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to reload cubes: {}", e).into(),
                            );
                        }
                    }
                }
                | Err(e) => {
                    web_sys::console::error_1(&format!("Failed to create cube: {}", e).into());
                }
            }
        });
    };

    // Filter state
    let filter_mode = RwSignal::new(FilterMode::Pinned);

    view! {
        <panel::Panel
            title="Glacier"
            resizable_right=true
            on_resize_right=on_resize_right.clone()
            on_double_click_resizer=on_double_click_resizer.clone()
            is_left_panel=is_left_panel
            header_actions=view! {
                <div class="flex items-center gap-1.5">
                    <FilterButton filter_mode=filter_mode.clone() />
                    <button::ActionButton
                        icon=view! { <Icon icon=LuPlus width="12" height="12" /> }
                        color_variant=ColorVariant::Primary
                        on_click=create_cube_action
                    />
                </div>
            }.into_any()
        >
            <card::CardList items=move || {
                let cubes_data = cubes.get();
                let mode = filter_mode.get();
                web_sys::console::log_1(
                    &format!("GlacierPanel: Got {} total cubes, filter mode: {:?}", cubes_data.len(), mode).into(),
                );
                let filtered: Vec<_> = cubes_data
                    .into_iter()
                    .filter(|cube| {
                        match mode {
                            FilterMode::Pinned => cube.inner.pin,
                            FilterMode::All => true,
                        }
                    })
                    .collect();
                web_sys::console::log_1(
                    &format!("GlacierPanel: {} cubes after filter", filtered.len()).into(),
                );
                filtered
                    .into_iter()
                    .map(|cube| view! { <CubeEntry cube=cube cubes=cubes.clone() /> }.into_any())
                    .collect::<Vec<_>>()
            } />
        </panel::Panel>
    }
}

#[component]
fn Workspace() -> impl IntoView {
    view! {
        <div
            class="flex flex-col h-full border-r overflow-hidden border-[var(--color-border)] bg-[var(--color-workspace-bg)]"
        >
            <div
                class="py-2 pl-3 pr-2 border-b border-[var(--color-border)] bg-[var(--color-panel-header-bg)]"
            >
                <h2 class="m-0 text-sm font-medium text-[var(--color-text)]">"Workspace"</h2>
            </div>
            <div class="flex-1 overflow-y-auto p-2">
                <div class="p-8 min-h-full">
                    <p class="italic text-center mt-12 text-[var(--color-text-muted)]">
                        "Select a cube from Glacier or drag raw material from Stream to start editing..."
                    </p>
                </div>
                </div>
        </div>
    }
}

#[component]
fn StreamPanel(
    raw_entries: RwSignal<Vec<Raw>>, open_settings: impl Fn(web_sys::MouseEvent) + 'static,
    on_resize_left: std::rc::Rc<dyn Fn(f64) + 'static>,
    on_double_click_resizer: std::rc::Rc<dyn Fn(bool) + 'static>,
    is_left_panel: bool,
) -> impl IntoView {
    view! {
        <panel::Panel
            title="Stream"
            resizable_left=true
            on_resize_left=on_resize_left.clone()
            on_double_click_resizer=on_double_click_resizer.clone()
            is_left_panel=is_left_panel
            header_actions=view! {
                <button::ActionButton
                    icon=view! { <Icon icon=LuSettings width="12" height="12" /> }
                    color_variant=ColorVariant::Secondary
                    on_click=open_settings
                />
            }.into_any()
        >
            <card::CardList items=move || {
                let entries = raw_entries.get();
                entries.into_iter()
                    .map(|raw| view! { <RawEntry raw=raw /> }.into_any())
                    .collect::<Vec<_>>()
            } />
        </panel::Panel>
    }
}

#[component]
fn CubeEntry(cube: Cube, cubes: RwSignal<Vec<Cube>>) -> impl IntoView {
    let title = match &cube.inner.content {
        | CubeContent::Graph(_) => "Graph".to_string(),
        | CubeContent::PreOrder(_) => "PreOrder".to_string(),
        | CubeContent::Order(_) => "Order".to_string(),
        | CubeContent::Point(_) => "Point".to_string(),
        | CubeContent::Meta => "Meta".to_string(),
    };
    let created = cube.created_at.format("%Y-%m-%d %H:%M").to_string();
    let cube_id = cube.id.0.to_string();
    
    let delete_action = {
        let cubes_signal = cubes.clone();
        let cube_id_clone = cube_id.clone();
        move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            let cubes_signal = cubes_signal.clone();
            let cube_id = cube_id_clone.clone();
            spawn_local(async move {
                web_sys::console::log_1(&format!("Deleting cube with id: {}", cube_id).into());
                match tauri_api::delete_cube(cube_id).await {
                    | Ok(_) => {
                        web_sys::console::log_1(&"Cube deleted successfully".into());
                        // Reload cubes
                        match tauri_api::get_all_cubes().await {
                            | Ok(cubes_data) => {
                                web_sys::console::log_1(
                                    &format!("Reloaded {} cubes", cubes_data.len()).into(),
                                );
                                cubes_signal.set(cubes_data);
                            }
                            | Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Failed to reload cubes: {}", e).into(),
                                );
                            }
                        }
                    }
                    | Err(e) => {
                        web_sys::console::error_1(&format!("Failed to delete cube: {}", e).into());
                    }
                }
            });
        }
    };
    
    view! {
        <div
            class="p-3 border rounded cursor-pointer transition-colors duration-200 hover-bg-panel border-[var(--color-border)] bg-[var(--color-panel-bg)] relative group"
        >
            <div class="flex items-start justify-between gap-2">
                <div class="flex-1">
                    <div class="font-medium mb-1 text-[var(--color-text)]">{title}</div>
                    <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{created}</div>
                </div>
                <button::ActionButton
                    icon=view! { <Icon icon=LuTrash2 width="12" height="12" /> }
                    color_variant=ColorVariant::Danger
                    on_click=delete_action
                    class="opacity-0 group-hover:opacity-100 transition-opacity duration-200 shrink-0"
                />
            </div>
        </div>
    }
}

#[component]
fn RawEntry(raw: Raw) -> impl IntoView {
    let source = format_raw_source(&raw.inner.source);
    let created = raw.created_at.format("%Y-%m-%d %H:%M").to_string();
    let content = raw.inner.content.clone();
    
    // Node ref for hover detection
    let element_ref = NodeRef::<leptos::html::Div>::new();
    
    // Use leptos_use hover utility
    let is_hovered = use_element_hover(element_ref);
    let preview_image_url = RwSignal::new(Option::<String>::None);
    
    // Only handle hover for image entries
    let is_image = matches!(content, RawContent::Image(_));
    
    // Get image dimensions for positioning calculations
    let image_dimensions = if let RawContent::Image(img) = &content {
        Some((img.width as f64, img.height as f64))
    } else {
        None
    };
    
    // Load full image on hover
    if is_image {
        let is_hovered_for_load = is_hovered.clone();
        let preview_image_url_for_load = preview_image_url.clone();
        let content_for_load = content.clone();
        
        Effect::new(move |_| {
            if is_hovered_for_load.get() {
                if let RawContent::Image(img) = &content_for_load {
                    let blob_id = img.blob_id;
                    let img_format = img.format.clone();
                    let preview_url = preview_image_url_for_load.clone();
                    
                    web_sys::console::log_1(
                        &format!("Loading preview for image blob: {}", blob_id.0).into(),
                    );
                    
                    spawn_local(async move {
                        match tauri_api::get_blob(blob_id.0.to_string()).await {
                            Ok(data) => {
                                web_sys::console::log_1(
                                    &format!("Loaded {} bytes for preview", data.len()).into(),
                                );
                                
                                // Convert blob data to base64 using the base64 crate
                                let base64 = general_purpose::STANDARD.encode(&data);

                                let format = img_format.as_deref().unwrap_or("jpeg");
                                let mime_type = match format {
                                    "png" => "image/png",
                                    "jpeg" | "jpg" => "image/jpeg",
                                    "gif" => "image/gif",
                                    "webp" => "image/webp",
                                    "svg" => "image/svg+xml",
                                    _ => "image/jpeg",
                                };
                                let data_url = format!("data:{};base64,{}", mime_type, base64);
                                web_sys::console::log_1(
                                    &format!("Created data URL with {} chars", data_url.len()).into(),
                                );
                                preview_url.set(Some(data_url));
                            }
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Failed to load image blob: {}", e).into(),
                                );
                            }
                        }
                    });
                }
            } else {
                // Clear preview when not hovering
                preview_image_url_for_load.set(None);
            }
        });
    }
    
    view! {
        <div
            node_ref=element_ref
            class="p-3 border rounded cursor-pointer transition-colors duration-200 w-full max-w-full box-border wrap-break-word hover-bg-panel border-[var(--color-border)] bg-[var(--color-panel-bg)] relative"
        >
            {move || {
                if is_image && is_hovered.get() {
                    if let Some(url) = preview_image_url.get() {
                        view! {
                            <div
                                class="fixed z-50 pointer-events-none"
                                style=move || {
                                    // Calculate position based on element position
                                    if let Some(element) = element_ref.get() {
                                        let rect = element.get_bounding_client_rect();
                                        
                                        // Get viewport dimensions
                                        let viewport_height = web_sys::window()
                                            .and_then(|w| w.inner_height().ok())
                                            .and_then(|h| h.as_f64())
                                            .unwrap_or(800.0);
                                        
                                        // Margin constant - same for all sides
                                        let margin = 16.0;
                                        
                                        // Entry center position
                                        let entry_center_y = rect.top() + (rect.height() / 2.0);
                                        
                                        // Calculate preview dimensions based on image aspect ratio
                                        let max_preview_size = 400.0;
                                        let (img_width, img_height) = image_dimensions.unwrap_or((max_preview_size, max_preview_size));
                                        let aspect_ratio = img_width / img_height;
                                        
                                        // Calculate preview dimensions maintaining aspect ratio
                                        let (preview_width, preview_height) = if aspect_ratio > 1.0 {
                                            // Landscape: width is limiting factor
                                            (max_preview_size, max_preview_size / aspect_ratio)
                                        } else {
                                            // Portrait or square: height is limiting factor
                                            (max_preview_size * aspect_ratio, max_preview_size)
                                        };
                                        
                                        // Check if preview is too tall for viewport and scale down if needed
                                        let available_height = viewport_height - (margin * 2.0); // margin top and bottom
                                        let (final_width, final_height) = if preview_height > available_height {
                                            // Scale down to fit height
                                            let scale = available_height / preview_height;
                                            (preview_width * scale, available_height)
                                        } else {
                                            (preview_width, preview_height)
                                        };
                                        
                                        // Calculate horizontal position (center preview with entry, positioned to the left)
                                        let left = rect.left() - final_width - margin;
                                        
                                        // Calculate vertical position (center preview with entry)
                                        let mut top = entry_center_y - (final_height / 2.0);
                                        
                                        // Define valid range with margins (same margin as horizontal)
                                        let min_top = margin;
                                        let max_top = viewport_height - final_height - margin;
                                        
                                        // Adjust if preview goes off screen - place at edge
                                        if top < min_top {
                                            // Would go off top - place at top edge with margin
                                            top = min_top;
                                        } else if top > max_top {
                                            // Would go off bottom - place at bottom edge with margin
                                            top = max_top;
                                        }
                                        
                                        // Final clamp to ensure it's always in bounds
                                        top = top.max(min_top).min(max_top);
                                        
                                        format!(
                                            "left: {}px; top: {}px; width: {}px; height: {}px;",
                                            left, top, final_width, final_height
                                        )
                                    } else {
                                        "display: none;".to_string()
                                    }
                                }
                            >
                                <img
                                    class="w-full h-full object-contain rounded border border-[var(--color-border)] shadow-lg bg-[var(--color-panel-bg)]"
                                    src=url
                                    alt="Preview"
                                />
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }
                } else {
                    view! { <></> }.into_any()
                }
            }}
            <RawPreview content=content />
            <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{source}</div>
            <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{created}</div>
        </div>
    }
}

#[component]
fn RawPreview(content: RawContent) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 mb-2">
            {match &content {
                RawContent::Text(text) => {
                    let preview_text = if text.content.len() > 50 {
                        format!("{}...", &text.content[..50])
                    } else {
                        text.content.clone()
                    };
                    view! {
                        <>
                            <div class="text-xl">"📄"</div>
                            <div class="flex-1 text-sm wrap-break-word text-[var(--color-text-secondary)]">{preview_text}</div>
                        </>
                    }.into_any()
                }
                RawContent::Image(img) => {
                    // Load thumbnail if available
                    let thumbnail_url = RwSignal::new(Option::<String>::None);
                    let thumbnail_url = thumbnail_url.clone();
                    let img = img.clone();

                    if let Some(thumb_blob_id) = img.thumbnail_blob_id {
                        web_sys::console::log_1(
                            &format!("Thumbnail exists for image: {}", thumb_blob_id.0).into(),
                        );
                        spawn_local(async move {
                            match tauri_api::get_blob(thumb_blob_id.0.to_string()).await {
                                Ok(data) => {
                                    // Convert blob data to base64 using the base64 crate
                                    let base64 = general_purpose::STANDARD.encode(&data);

                                    let format = img.format.as_deref().unwrap_or("jpeg");
                                    let mime_type = match format {
                                        "png" => "image/png",
                                        "jpeg" | "jpg" => "image/jpeg",
                                        "gif" => "image/gif",
                                        "webp" => "image/webp",
                                        "svg" => "image/svg+xml",
                                        _ => "image/jpeg",
                                    };
                                    let data_url = format!("data:{};base64,{}", mime_type, base64);
                                    thumbnail_url.set(Some(data_url));
                                }
                                Err(_) => {
                                    // Fallback to emoji on error
                                }
                            }
                        });
                    } else {
                        web_sys::console::log_1(
                            &"Thumbnail does not exist for image, using fallback emoji".into(),
                        );
                    }

                    view! {
                        <>
                            <div class="text-xl w-5 h-5 flex items-center justify-center shrink-0">
                                    {move || {
                                        if let Some(url) = thumbnail_url.get() {
                                        view! { <img class="w-5 h-5 object-cover rounded shrink-0" src=url alt="Thumbnail" /> }.into_any()
                                        } else {
                                            view! { <span>"🖼️"</span> }.into_any()
                                        }
                                    }}
                                </div>
                            <div class="flex-1 text-sm wrap-break-word text-[var(--color-text-secondary)]">{format!("Image ({}x{})", img.width, img.height)}</div>
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

fn format_raw_source(source: &RawSource) -> String {
    match source {
        | RawSource::Clipboard { application } => {
            format!(
                "Clipboard{}",
                application.as_ref().map(|a| format!(" ({})", a)).unwrap_or_default()
            )
        }
        | RawSource::FileWatcher { original_path } => {
            format!("File: {}", original_path.display())
        }
        | RawSource::ManualImport => "Manual Import".to_string(),
    }
}
