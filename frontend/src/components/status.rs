use gloo_net::http::Request;
use models::{SystemStatus, SystemStatusResponse};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;


#[function_component(Status)]
pub fn status() -> Html {
    let data = use_state(|| None);

    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/status").send().await.unwrap();
                    let result: Result<SystemStatusResponse, String> = {
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
            <h3>{ "System Status" }</h3>
            {

            match data.as_ref() {
                Some(Ok(data)) => {
                    if data.overall == SystemStatus::Healthy {
                        html!{
                            <>
                                <div class="pure-button pure-button-success" style="background: rgb(28, 184, 65);">{" System status ok "}</div>
                                <pre>
                                    { data.maven_output.clone() }
                                </pre>
                            </>
                        }
                    } else {
                        html!{
                            <>
                                <div class="pure-button pure-button-success" style="background: rgb(202, 60, 60);">{" System status not ok "}</div>
                                <pre>
                                    { data.maven_output.clone() }
                                </pre>
                            </>
                        }
                    }
                },
                None => {
                    html!{
                        <p>{ "Loading..." }</p>
                    }
                }
                _ => { html!{} }
            }
        }
        </article>
    }
}