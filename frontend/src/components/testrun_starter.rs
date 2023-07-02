use std::{collections::HashMap, hash::Hash, sync::Arc};

use gloo_net::http::Request;
use log::info;
use models::{config::AppConfig, RunTestParam};
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

    let mut ref_map : HashMap<&str, NodeRef> = HashMap::new();
    /*
    if let Some(config) = data.as_ref() {
        for p in &config.simulation.params {
            ref_map.insert("URL", use_node_ref());
        }
    } */
    ref_map.insert("URL", use_node_ref());
    ref_map.insert("DURATION", use_node_ref());
    ref_map.insert("FACTOR", use_node_ref());
    ref_map.insert("SCENARIO", use_node_ref());

    let ref_map = Arc::new(ref_map);
    let description = use_node_ref();

    let onsubmit = {
        let ref_map = ref_map.clone();
        let description = description.clone();
        let message = message.clone();

        Callback::from(move |ev: SubmitEvent| {
            ev.prevent_default();

            let description = description
                .cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse()
                .unwrap();
            let message = message.clone();

            let mut custom_params : HashMap<String, String> = HashMap::new();
            for x in ref_map.as_ref() {
                let y : String = x.1.cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse()
                .unwrap();
                custom_params.insert(String::from(*x.0), y);
            }

            wasm_bindgen_futures::spawn_local(async move {
                let body = RunTestParam {
                    description: description,
                    custom_params,
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
                {
                    ref_map.iter().map(|e|
                        html!(
                        <div class="pure-control-group">
                        <label for="scenario">{e.0}</label>
                        <input ref={e.1} id={*e.0}  value={ data.as_ref().and_then(|d| d.get_param(e.0)).unwrap_or("default".into()) }/>
                        </div>
                        )
                    ).collect::<Html>()
                }
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
