use components::compare_view::CompareView;
use components::testrun_list::TestrunList;
use components::testrun_show::TestrunShow;
use components::testrun_starter::TestrunStarter;
use yew::prelude::*;
use yew_router::prelude::*;

pub mod components;
pub mod store;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <main>
                <img src="/water-droplet.svg" width="40" style="float: left; padding-right: 1em; padding-left: 1em;" />
                <h1 class="h1">{ "Waterpistol" }</h1>
                <TestrunStarter />
                <TestrunList />
                <TestrunShow />
                <CompareView />
            </main>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
