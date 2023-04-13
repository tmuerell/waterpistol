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

    let description = use_node_ref();
    let duration = use_node_ref();
    let factor = use_node_ref();
    let scenario = use_node_ref();
    let url = use_node_ref();

    let onsubmit = {
        let description = description.clone();
        let duration = duration.clone();
        let factor = factor.clone();
        let scenario = scenario.clone();
        let url = url.clone();
        let message = message.clone();

        Callback::from(move |ev: SubmitEvent| {
            ev.prevent_default();

            let description = description
                .cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse()
                .unwrap();
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
                    description: description,
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
            <form {onsubmit} class="pure-form pure-form-aligned">
                <div class="pure-control-group">
                    <label for="description">{"Description"}</label>
                    <input ref={description} id="description" class="pure-input-1-2" />
                </div>
                <div class="pure-control-group">
                    <label for="url">{"URL"}</label>
                    <input ref={url} id="url"  value={ data.as_ref().and_then(|d| d.get_param("BASE_URL")).unwrap_or("https://example.com".into()) } class="pure-input-1-2"/>
                </div>
                <div class="pure-control-group">
                    <label for="duration">{"Duration"}</label>
                    <input ref={duration} id="duration" value={ data.as_ref().and_then(|d| d.get_param("DURATION")).unwrap_or("60".into()) }/>
                </div>
                <div class="pure-control-group">
                    <label for="factor">{"Factor"}</label>
                    <input ref={factor} id="factor" value={ data.as_ref().and_then(|d| d.get_param("FACTOR")).unwrap_or("1".into()) }/>
                </div>
                <div class="pure-control-group">
                    <label for="scenario">{"Scenario"}</label>
                    <input ref={scenario} id="scenario"  value={ data.as_ref().and_then(|d| d.get_param("SCENARIO")).unwrap_or("default".into()) }/>
                </div>
                <div class="pure-controls">
                    <button type="submit" class="pure-button pure-button-primary">{ "Start gatling run" }</button>
                </div>
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
