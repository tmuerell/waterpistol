use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[function_component(Navigation)]
pub fn navigation() -> Html {
    let navigator = use_navigator().unwrap();
    let onclick = Callback::from(move |ev : MouseEvent| { ev.prevent_default(); navigator.push(&Route::Home) });

    let navigator = use_navigator().unwrap();
    let onclick_testsuites = Callback::from(move |ev : MouseEvent| { ev.prevent_default(); navigator.push(&Route::Testsuites) });

    let navigator = use_navigator().unwrap();
    let onclick_status = Callback::from(move |ev : MouseEvent| { ev.prevent_default(); navigator.push(&Route::Status) });
    html! {
        <div class="pure-menu pure-menu-horizontal">
        <ul class="pure-menu-list">
            <li class="pure-menu-item">
                <a {onclick} href="#" class="pure-menu-link">{ "Home" }</a>
            </li>
            <li class="pure-menu-item">
                <a onclick={onclick_testsuites} href="#" class="pure-menu-link">{ "Testsuites" }</a>
            </li>
            <li class="pure-menu-item">
                <a onclick={onclick_status} href="#" class="pure-menu-link">{ "Status" }</a>
            </li>
        </ul>
        </div>
    }
}