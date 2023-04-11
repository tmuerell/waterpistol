use gloo_net::http::Request;
use log::info;
use models::{RunTestParam, config::AppConfig};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(TestrunStarter)]
pub fn testrun_starter() -> Html {
    let message: UseStateHandle<Option<String>> = use_state(|| None);
    let data = use_state(|| None);
    let _current_factor = use_state(|| 1u64);

    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/config").send().await.unwrap();
                    let result: Result<AppConfig, String> = {
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
                    data.set(Some(result.unwrap()));
                });
            }

            || {}
        });
    }

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

            let duration = duration
                .cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse()
                .unwrap();
            let factor = factor
                .cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse()
                .unwrap();
            let scenario = scenario.cast::<HtmlInputElement>().unwrap().value();
            let url = url.cast::<HtmlInputElement>().unwrap().value();
            let message = message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let body = RunTestParam {
                    duration: duration,
                    factor: factor,
                    scenario: scenario,
                    url: url,
                };
                let _ = Request::post("/api/run")
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                    .unwrap();
                message.set(Some("Run was started.".to_string()))
            });
        })
    };

    html! {
        <article>
            <h3>{"Start run"}</h3>
            <form {onsubmit} class="pure-form">
                <input ref={duration} placeholder="Duration" value={ data.as_ref().and_then(|d| d.get_param("DURATION")).unwrap_or("60".into()) }/>
                <input ref={factor} placeholder="Factor"  value={ data.as_ref().and_then(|d| d.get_param("FACTOR")).unwrap_or("1".into()) }/>
                <input ref={scenario} placeholder="Scenario"  value={ data.as_ref().and_then(|d| d.get_param("SCENARIO")).unwrap_or("default".into()) }/>
                <input ref={url} placeholder="Url"  value={ data.as_ref().and_then(|d| d.get_param("URL")).unwrap_or("https://example.com".into()) }/>

                <button type="submit" class="pure-button pure-button-primary">{ "Start gatling run" }</button>
            </form>
            {
                if message.is_some() {
                    html! {
                        <p style="color: green;">{ message.as_ref().unwrap() }</p>
                    }
                } else {
                    html!()
                }
            }
        </article>
    }
}
