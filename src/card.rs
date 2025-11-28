use leptos::prelude::*;
use styled::style;

#[component]
pub fn CardList<F>(items: F) -> impl IntoView
where
    F: Fn() -> Vec<AnyView> + 'static + Send,
{
    let styles = style! {
        .card-list {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }
    };
    styled::view! { styles,
        <div class="card-list">
            {move || items()}
        </div>
    }
}

