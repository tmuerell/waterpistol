use gloo_net::http::Request;
use models::Testrun;

use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};
use yewdux::prelude::use_store;

use crate::store::{CompareSelection, TestrunDataSelection};

#[function_component(TestrunList)]
pub fn testrun_list() -> Html {
    let data = use_state(|| None);
    let selected_testruns = use_state(|| vec![]);
    let (_selection, dispatch) = use_store::<TestrunDataSelection>();

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

    let onclick = {
        let selected_testruns = selected_testruns.clone();
        let (_, dispatch2) = use_store::<CompareSelection>();

        Callback::from(move |_ev: MouseEvent| {
            dispatch2.set(CompareSelection {
                testrun_data: Some(selected_testruns.to_vec()),
            })
        })
    };

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
                                let d = dispatch.clone();
                                let onclick = Callback::from(move |_ev:MouseEvent| {
                                    d.set(TestrunDataSelection { testrun_data: x.clone() });
                                });

                                let selected_testruns = selected_testruns.clone();
                                let x = testrun.data.clone();
                                let onchange = Callback::from(move |ev:Event| {
                                    let input = ev
                                    .target()
                                    .unwrap()
                                    .dyn_into::<web_sys::HtmlInputElement>()
                                    .unwrap();

                                    if input.checked() {
                                        let mut temp: Vec<_> = selected_testruns.to_vec();
                                        temp.push(x.clone().unwrap());
                                        selected_testruns.set(temp);
                                    } else {
                                        if let Some(ref x) = x {
                                            let mut temp: Vec<_> = selected_testruns.to_vec();
                                            temp.retain(|a| a.datum != x.datum);
                                            selected_testruns.set(temp);
                                        }
                                    }
                                });


                                let (total, nok_ratio) = if let Some(ref st) = testrun.data.as_ref().unwrap().statistics {
                                    (st.requests_nok + st.requests_ok, st.requests_nok as f32 / (st.requests_nok as f32 +st.requests_ok as f32))
                                } else {
                                    (0, 0.0f32)
                                };
                                html!{

                                <tr key={testrun.name.clone()}>
                                    <td>
                                        <input type="checkbox" {onchange}/>
                                    </td>
                                    <td>{ testrun.data.as_ref().and_then(|x| x.datum).map(|x| x.format("%Y-%m-%d %H:%M")) }</td>
                                    <td>{ testrun.data.as_ref().and_then(|x| x.statistics.as_ref()).map(|x| x.name.clone()).unwrap_or("---".into()) }</td>
                                    <td>{ format!("{:?}", testrun.data.as_ref().unwrap().status) }</td>
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
