use gloo_net::http::Request;
use models::Testsuite;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;


#[function_component(TestsuiteList)]
pub fn testsuite_list() -> Html {
    let data = use_state(|| None);

    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/testsuites").send().await.unwrap();
                    let result: Result<Vec<Testsuite>, String> = {
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

    html! {
        <article>
            <h3>{ "Testsuite list" }</h3>
            <table class="pure-table">
                <thead>
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Type" }</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                {
                    match data.as_ref() {
                        Some(Ok(data)) => {
                            data.iter().map(|testsuite| {
                                html! {
                                    <tr>
                                        <td>{ testsuite.name.clone() }</td>
                                        <td>{ "Gatling" }</td>
                                        <td>{ "Active" }</td>
                                    </tr>
                                }
                            }).collect::<Html>()},
                        _ => { html!{} }
                    }
                }
                </tbody>
            </table>
        </article>
    }
}