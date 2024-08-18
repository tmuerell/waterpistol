use std::collections::{BTreeMap, HashMap};

use gloo_net::http::Request;
use models::{config::AppConfig, RunTestParam};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props;

pub enum Message {
    Submit,
    ConfigData(AppConfig),
}

pub struct Ignition {
    description: NodeRef,
    message: Option<String>,
    properties: BTreeMap<String, NodeRef>,
    data: Option<AppConfig>,
}

impl Component for Ignition {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Ignition {
            description: NodeRef::default(),
            properties: BTreeMap::new(),
            message: None,
            data: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Submit => {
                let description = self
                    .description
                    .cast::<HtmlInputElement>()
                    .unwrap()
                    .value()
                    .parse()
                    .unwrap();

                let mut custom_params: HashMap<String, String> = HashMap::new();
                for x in &self.properties {
                    let y: String =
                        x.1.cast::<HtmlInputElement>()
                            .unwrap()
                            .value()
                            .parse()
                            .unwrap();
                    custom_params.insert(String::from(x.0), y);
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
                });

                self.message = Some("Run was started.".to_string());

                true
            }
            Message::ConfigData(data) => {
                let mut ref_map: BTreeMap<String, NodeRef> = BTreeMap::new();
                for d in &data.simulation.params {
                    ref_map.insert(d.name.clone(), NodeRef::default());
                }

                self.data = Some(data);
                self.properties = ref_map;

                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
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
                link.send_message(Message::ConfigData(result.unwrap()));
            });
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.link().callback(|ev : SubmitEvent| { ev.prevent_default(); Message::Submit});
        html! {
            <article>
                <h3>{"Start run"}</h3>
                <form {onsubmit} class="pure-form pure-form-aligned">
                    <div class="pure-control-group">
                        <label for="description">{"Description"}</label>
                        <input ref={self.description.clone()} id="description" class="pure-input-1-2" />
                    </div>
                    {
                        if let Some(ref d) = self.data {
                            self.properties.iter().map(|e|
                                html!(
                                <div class="pure-control-group">
                                <label for="scenario">{e.0}</label>
                                <input ref={e.1} id={e.0.clone()}  value={ d.get_param(e.0).unwrap_or("default".into()) }  class="pure-input-1-2"/>
                                </div>
                                )
                            ).collect::<Html>()
                        } else {
                            html! {
                                <p style="color: gray;">{ "Please wait" }</p>
                            }
                        }
                    }
                    <div class="pure-controls">
                        <button type="submit" class="pure-button pure-button-primary">{ "Start gatling run" }</button>
                    </div>
                </form>
                {
                    if let Some(ref m) = self.message {
                        html! {
                            <p style="color: green;">{ m }</p>
                        }
                    } else {
                        html!()
                    }
                }
            </article>
        }
    }
}
