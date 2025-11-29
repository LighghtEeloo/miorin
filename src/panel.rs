use leptos::prelude::*;
use styled::style;

pub fn panel_styles() -> Result<styled::Style, stylist::Error> {
    style! {
        .panel {
            display: flex;
            flex-direction: column;
            height: 100%;
            max-height: 100%;
            min-height: 0;
            border-right: 1px solid var(--color-border);
            background-color: var(--color-panel-bg);
            overflow: auto;
        }
        .panel:last-child {
            border-right: none;
        }
    }
}

pub fn panel_header_styles() -> Result<styled::Style, stylist::Error> {
    style! {
        .panel-header {
            flex-shrink: 0;
            padding: 1rem;
            border-bottom: 1px solid var(--color-border);
            background-color: var(--color-panel-header-bg);
        }
        .panel-header h2 {
            margin: 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--color-text);
        }
    }
}

pub fn panel_header_with_actions_styles() -> Result<styled::Style, stylist::Error> {
    style! {
        .panel-header {
            flex-shrink: 0;
            padding: 1rem;
            border-bottom: 1px solid var(--color-border);
            background-color: var(--color-panel-header-bg);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .panel-header h2 {
            margin: 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--color-text);
        }
        .settings-button {
            padding: 6px;
            background: var(--color-secondary, #6c757d);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            width: 28px;
            height: 28px;
            min-width: 28px;
            transition: background-color 0.2s;
        }
        .settings-button:hover {
            background-color: var(--color-secondary-hover, #5a6268);
        }
    }
}

pub fn panel_content_styles() -> Result<styled::Style, stylist::Error> {
    style! {
        .panel-content {
            display: flex;
            flex-direction: column;
            flex: 1 1 0;
            min-height: 0;
            // overflow-y: auto;
            // overflow-x: hidden;
            overflow: auto;
            padding: 0.5rem;
            box-sizing: border-box;
        }
    }
}

#[component]
pub fn Panel(
    title: &'static str, #[prop(optional)] header_actions: Option<AnyView>,
    children: Children,
) -> impl IntoView {
    let panel_wrapper_styles = style! {
        .panel-wrapper {
            height: 100vh;
            max-height: 100%;
            min-height: 0;
            overflow: auto;
            display: flex;
            flex-direction: column;
            box-sizing: border-box;
        }
    };

    let has_actions = header_actions.is_some();
    let header_styles =
        if has_actions { panel_header_with_actions_styles() } else { panel_header_styles() };

    let inner_panel = styled::view! { panel_styles(),
        <div class="panel">
            {styled::view! { header_styles,
                <div class="panel-header">
                    <h2>{title}</h2>
                    {header_actions}
                </div>
            }}
            {styled::view! { panel_content_styles(),
                <div class="panel-content">
                    {children()}
                </div>
            }}
        </div>
    };

    styled::view! { panel_wrapper_styles,
        <div class="panel-wrapper">
            {inner_panel}
        </div>
    }
    .into_any()
}
