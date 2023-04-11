use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::store::TestrunDataSelection;


#[function_component(TestrunShow)]
pub fn testrun() -> Html {
    let (selection, _dispatch) = use_store::<TestrunDataSelection>();

    match selection.testrun_data {
        Some(ref tr) =>     {
            let s = tr.statistics.as_ref().unwrap();
        html! {
            <article>
                    <h5>{ format!("{}", s.name)}</h5>

                    <div class="pure-g">

                    <div class="pure-u-1-2">
                        <table class="pure-table">
                            <thead>
                            <tr>
                                <th>{"Request"}</th>
                                <th>{"Count"}</th>
                                <th>{"OK"}</th>
                                <th>{"Min"}</th>
                                <th>{"Max"}</th>
                                <th>{"Avg"}</th>
                                <th>{"P95"}</th>
                            </tr>
                            </thead>
                            <tbody> {
                                s.request_stats.iter().map(|x| {
                                    let errors : u64 = x.errors.iter().map(|x| x.count).sum();
                                    html!{
                                        <tr>
                                        <td>{ format!("{}", x.name) }</td>
                                        <td>{ format!("{}", x.count) }</td>
                                        <td>{ format!("{}", x.count - errors) }</td>
                                        <td>{ format!("{}", x.min) }</td>
                                        <td>{ format!("{}", x.max) }</td>
                                        <td>{ format!("{}", x.avg) }</td>
                                        <td>{ format!("{}", x.p95) }</td>
                                        </tr>
                                    }

                                }).collect::<Html>()
                            }
                            </tbody>
                        </table>
                    </div>
                    <div class="pure-u-1-2">
                    <table class="pure-table">
                            <thead>
                            <tr>
                                <th>{"Journey"}</th>
                                <th>{"Count"}</th>
                            </tr>
                            </thead>
                            <tbody> {
                                s.user_stats.iter().map(|x| {
                                    html!{
                                        <tr>
                                        <td>{ format!("{}", x.name) }</td>
                                        <td>{ format!("{}", x.count) }</td>
                                        </tr>
                                    }

                                }).collect::<Html>()
                            }
                            </tbody>
                        </table>
                    <p>{ format!("Min: {:?}", tr)}</p>
                    </div>
                </div>
                </article>
        }},
        None => html!()
    }
}