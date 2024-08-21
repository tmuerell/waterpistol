use components::compare_view::CompareView;
use components::ignition::Ignition;
use components::status::Status;
use components::testrun_list::TestrunList;
use components::testrun_show::TestrunShow;
use components::testsuite_list::TestsuiteList;
use components::uploader::Uploader;
use components::navigation::Navigation;
use yew::prelude::*;
use yew_router::prelude::*;

pub mod components;
pub mod store;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/testsuites")]
    Testsuites,
    #[at("/status")]
    Status,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    html! {
            <main>
            <div class="pure-g">
                <div class="pure-u-2-3">
                    <div style="padding: 1em;">
                        <img src="/water-droplet.svg" width="40" style="float: left; padding: 1em;" />
                        <h1 class="h1">{ "Waterpistol" }</h1>
                    </div>
                </div>
                <div class="pure-u-1-3">
                    <div style="padding: 2em;">
                        <Navigation />
                    </div>
                </div>
            </div>
            {
            match routes {
                Route::Home => html! {
                    <>
                        <Ignition />
                        <TestrunList />
                        <TestrunShow />
                        <CompareView />
                    </>
                },
                Route::NotFound => html! {
                    <>
                        { "This page cannot be found" }
                    </>
                },
                Route::Testsuites => html! {
                    <>
                        <Uploader />
                        <TestsuiteList />
                    </>
                },
                Route::Status => html! {
                    <>
                        <Status />
                    </>
                },
            }
        }
        </main>
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
