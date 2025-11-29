use leptos::prelude::*;
use styled::style;

#[component]
pub fn CardList<F>(items: F) -> impl IntoView
where
    F: Fn() -> Vec<AnyView> + 'static + Send,
{
    let styles = style! {
        .card-list {
            border: 3px solid green;
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
            flex: 1 1 auto;
            width: 100%;
            max-width: 100%;
            min-width: 0;
            min-height: 0;
            max-height: 100%;
            box-sizing: border-box;
            // overflow-y: auto;
            // overflow-x: hidden;
            overflow: auto;
        }
    };
    styled::view! { styles,
        <div class="card-list">
            {move || items()}
        </div>
    }
}

