use std::{collections::HashMap, hash::Hash, sync::Arc};

use gloo_net::http::Request;
use log::info;
use models::{config::AppConfig, RunTestParam};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;


enum Msg {
    FormSubmit,
    Message(Option<String>)
}

#[derive(Default, Properties, PartialEq)]
struct ParentProps {
    app_data : AppConfig
}

#[derive(Debug, Clone)]
struct TestrunStarter2 {
    description : NodeRef,
    refs: HashMap<String, NodeRef>,
    message : Option<String>,
}

impl Component for TestrunStarter2 {

    type Message = Msg;
    type Properties = ParentProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut m : HashMap<String, NodeRef> = HashMap::new();
        for p in &ctx.props().app_data.simulation.params {
            m.insert(String::from(&p.name), NodeRef::default());
        }
        Self {
            description: NodeRef::default(),
            refs: m,
            message: None
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link();
        match msg {
            Msg::FormSubmit => {
                let description = self.description
                .cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse()
                .unwrap();

            let mut custom_params : HashMap<String, String> = HashMap::new();
            for x in &self.refs {
                let y : String = x.1.cast::<HtmlInputElement>()
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
                link.send_message(Msg::Message(Some("Run was started".to_string())));
            });
            true
            },
            Msg::Message(x) => {
                self.message = x;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let data = &ctx.props().app_data;
        let onsubmit = link.callback(|_| Msg::FormSubmit);
        let fields = self.refs.into_iter().map(|e|
            html!(
            <div class="pure-control-group">
            <label for="scenario">{&e.0}</label>
            <input ref={e.1.clone()} id={e.0.clone()}  value={ data.get_param(&e.0).unwrap_or("default".into()) }/>
            </div>
            )
        ).collect::<Html>();
        html! {
            <article>
                <h3>{"Start run"}</h3>
                <form {onsubmit} class="pure-form pure-form-aligned">
                    <div class="pure-control-group">
                        <label for="description">{"Description"}</label>
                        <input ref={&self.description} id="description" class="pure-input-1-2" />
                    </div>
                    { fields }
                    <div class="pure-controls">
                        <button type="submit" class="pure-button pure-button-primary">{ "Start gatling run" }</button>
                    </div>
                </form>
                {
                    if self.message.is_some() {
                        html! {
                            <p style="color: green;">{ self.message.as_ref().unwrap() }</p>
                        }
                    } else {
                        html!()
                    }
                }
            </article>
        }
    }
}