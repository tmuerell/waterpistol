use std::vec;

use gloo_net::http::Request;
use models::report::TestrunStatus;
use models::{report::TestrunData, Testrun};
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::{html, Component, Html, Properties};
use yew::{platform::spawn_local, prelude::*};
use yewdux::prelude::{use_store, Dispatch};

use crate::store::{CompareSelection, TestrunDataSelection};

pub enum Msg {
    Selected(Option<TestrunData>),
    Unselected(Option<TestrunData>),
    Clicked(TestrunDataSelection),
    Changed,
    Compare,
    Data(Result<Vec<Testrun>, String>),
    Refresh,
}

#[derive(Properties, PartialEq)]
pub struct Props {}

pub struct TestrunList {
    pub data: Option<Result<Vec<Testrun>, String>>,
    pub selected_testruns: Vec<TestrunData>,
}

impl Component for TestrunList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        TestrunList {
            data: None,
            selected_testruns: vec![],
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        match self.data {
            None => html! {
                <div>{"No server response"}</div>
            },
            Some(Ok(ref data)) => {
                let onclick = ctx.link().callback(|_| Msg::Compare);
                let onclick2 = ctx.link().callback(|_| Msg::Refresh);
                html! {
                    <article>
                        <h3>{"Testruns"}</h3>
                        <div style="max-height: 400px; overflow: auto;">
                        <table class="pure-table">
                        <thead>
                        <tr>
                        <th></th>
                        <th>{ "Date" }</th>
                        <th>{ "Name" }</th>
                        <th>{ "Status" }</th>
                        <th>{ "Scenario" }</th>
                        <th>{ "Duration" }</th>
                        <th>{ "Factor" }</th>
                        <th>{ "Requests" }</th>
                        <th>{ "(Failure%)" }</th>
                        <th></th>
                        </tr>
                        </thead>
                        <tbody>
                        {
                            {
                                data.iter().map(|testrun| {
                                    let x = testrun.data.clone();
                                    let onclick = ctx.link().callback(move |_| Msg::Clicked(TestrunDataSelection { testrun_data: x.clone() }));
                                    let x = testrun.data.clone();
                                    let onchange = ctx.link().callback(move |ev:Event| {
                                        let input = ev
                                        .target()
                                        .unwrap()
                                        .dyn_into::<web_sys::HtmlInputElement>()
                                        .unwrap();

                                        if input.checked() {
                                            Msg::Selected(x.clone())
                                        } else {
                                            Msg::Unselected(x.clone())
                                        }
                                    }
                                    );

                                    let (total, nok_ratio) = if let Some(ref st) = testrun.data.as_ref().unwrap().statistics {
                                        (st.requests_nok + st.requests_ok, st.requests_nok as f32 / (st.requests_nok as f32 +st.requests_ok as f32))
                                    } else {
                                        (0, 0.0f32)
                                    };

                                    let row_class = match testrun.data.as_ref().map(|e| &e.status) {
                                        Some(TestrunStatus::Running) => {
                                            "running"
                                        },
                                        _ => ""
                                    };
                                    let progress_text = if let Some(progress) = testrun.progress {
                                        format!(" ({} Users)", progress)
                                    } else {
                                        "".into()
                                    };
                                    html!{

                                    <tr key={testrun.name.clone()} class={row_class}>
                                        <td>
                                            <input type="checkbox" {onchange}/>
                                        </td>
                                        <td>{ testrun.data.as_ref().and_then(|x| x.datum).map(|x| x.format("%Y-%m-%d %H:%M")) }</td>
                                        <td>{ testrun.data.as_ref().and_then(|x| x.statistics.as_ref()).map(|x| x.name.clone()).unwrap_or("---".into()) }</td>
                                        <td>{ format!("{:?}", testrun.data.as_ref().unwrap().status) } {progress_text}</td>
                                        <td>{ format!("{}", testrun.data.as_ref().unwrap().custom_params.get("SCENARIO").unwrap_or(&"---".to_owned())) }</td>
                                        <td>{ format!("{}", testrun.data.as_ref().unwrap().custom_params.get("DURATION").unwrap_or(&"---".to_owned())) }</td>
                                        <td>{ format!("{}", testrun.data.as_ref().unwrap().custom_params.get("FACTOR").unwrap_or(&"---".to_owned())) }</td>
                                        <td>{ format!("{}", total)}</td>
                                        <td>{ format!("{:.4}%", nok_ratio*100.0)}</td>
                                        <td>
                                            <button {onclick} class="pure-button">{ "show" }</button>
                                            <a href={format!("/simulations/{}/", testrun.name)} class="pure-button" target="_blank">{ "report" }</a>
                                        </td>
                                    </tr>
                                    }
                                }).collect::<Html>()
                            }
                        }
                        </tbody>
                        </table>
                        </div>
                        <button {onclick} class="pure-button pure-button-primary">{ "Compare" }</button>
                        <button onclick={onclick2} class="pure-button pure-button-primary">{ "Refresh" }</button>
                    </article>
                }
            }
            Some(Err(ref err)) => {
                html! {
                    <div>{"Error requesting data from server: "}{err}</div>
                }
            }
        }
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
        if first_render && self.data.is_none() {
            self.update_list(ctx);
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Data(d) => {
                self.data = Some(d);
                true
            }
            Msg::Refresh => {
                self.update_list(ctx);
                false
            }
            Msg::Selected(Some(d)) => {
                self.selected_testruns.push(d);
                true
            }
            Msg::Unselected(Some(d)) => {
                self.selected_testruns.retain(|a| a.datum != d.datum);
                true
            }
            Msg::Clicked(d) => {
                let dispatch = Dispatch::<TestrunDataSelection>::new();
                dispatch.set(d);
                true
            }
            Msg::Compare => {
                let dispatch = Dispatch::<CompareSelection>::new();
                dispatch.set(CompareSelection {
                    testrun_data: Some(self.selected_testruns.clone()),
                });
                true
            }
            _ => false,
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn prepare_state(&self) -> Option<String> {
        None
    }
}

impl TestrunList {
    fn update_list(&mut self, ctx: &yew::Context<Self>) {
        let link = ctx.link().clone();
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
            link.send_message(Msg::Data(result));
        });
    }
}
