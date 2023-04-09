use models::{RunTestParam, Testrun};
use gloo_net::http::Request;
use log::info;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

use yew::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <main class="container">
                <h1>{ "Waterpistol" }</h1>
                <TestrunStarter />
                <TestrunList />
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

#[function_component(TestrunStarter)]
fn testrun_starter() -> Html {
    let message : UseStateHandle<Option<String>> = use_state(|| None);
    let current_factor = use_state(|| 1u64);

    let duration = use_node_ref();
    let factor = use_node_ref();
    let scenario = use_node_ref();
    let url = use_node_ref();

    let onsubmit = {
        let duration = duration.clone();
        let factor = factor.clone();
        let scenario = scenario.clone();
        let url = url.clone();
        let message = message.clone();

        Callback::from(move |ev: SubmitEvent| {
            ev.prevent_default();

            let duration = duration.cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
            let factor = factor.cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
            let scenario = scenario.cast::<HtmlInputElement>().unwrap().value();
            let url = url.cast::<HtmlInputElement>().unwrap().value();
            let message = message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let body = RunTestParam {
                    duration: duration,
                    factor: factor,
                    scenario: scenario,
                    url: url
                };
                let resp = Request::post("/api/run")
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                    .unwrap();
                info!("{}", resp.text().await.unwrap());
                message.set(Some("Run was started.".to_string()))
            });
        })
    };

    html! {
        <article>
            <h3>{"Start run"}</h3>
            <form {onsubmit}>
                <input ref={duration} placeholder="Duration" value="60"/>
                <input ref={factor} placeholder="Factor"  value="1"/>
                <input ref={scenario} placeholder="Scenario"  value="default"/>
                <input ref={url} placeholder="Url"  value="https://example.com/"/>

                <button type="submit">{ "Click" }</button>
            </form>
            <p>{ format!("{:?}", message) }</p>
        </article>
    }
}

#[function_component(TestrunList)]
fn testrun_list() -> Html {
    let data = use_state(|| None);

    // Request `/api/hello` once
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/testruns").send().await.unwrap();
                    let result: Result<Vec<Testrun>, String> = {
                        if !resp.ok() {
                            Err(format!(
                                "Error fetching data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                        } else {
                            resp.json().await.map_err(|err| err.to_string())
                        }
                    };
                    data.set(Some(result));
                });
            }

            || {}
        });
    }

    match data.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <article>
                    <h3>{"Testruns"}</h3>
                    <table>
                    <thead>
                    <tr>
                    <th>{ "Date" }</th>
                    <th>{ "Name" }</th>
                    <th>{ "Status" }</th>
                    <th>{ "Scenario" }</th>
                    <th>{ "Duration" }</th>
                    <th>{ "Factor" }</th>
                    <th>{ "Requests" }</th>
                    <th>{ "(Failure%)" }</th>
                    </tr>
                    </thead>
                    <tbody>
                    {
                        {
                            data.iter().map(|testrun| {
                                let (total, nok_ratio) = if let Some(ref st) = testrun.data.as_ref().unwrap().statistics {
                                    (st.requests_nok + st.requests_ok, st.requests_nok as f32 / (st.requests_nok as f32 +st.requests_ok as f32))
                                } else {
                                    (0, 0.0f32)
                                };
                                html!{

                                <tr key={testrun.name.clone()}>
                                    <td>{ testrun.creation_date.clone() }</td>
                                    <td><a href={format!("/simulations/{}/", testrun.name)}>{ format!("{}",testrun.name) }</a></td>
                                    <td>{ format!("{:?}", testrun.data.as_ref().unwrap().status) }</td>
                                    <td>{ format!("{}", testrun.data.as_ref().unwrap().scenario) }</td>
                                    <td>{ format!("{}", testrun.data.as_ref().unwrap().duration) }</td>
                                    <td>{ format!("{}", testrun.data.as_ref().unwrap().factor) }</td>
                                    <td>{ format!("{}", total)}</td>
                                    <td>{ format!("{:.4}%", nok_ratio*100.0)}</td>
                                </tr>
                                }
                            }).collect::<Html>()
                        }
                    }
                    </tbody>
                    </table>
                </article>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
