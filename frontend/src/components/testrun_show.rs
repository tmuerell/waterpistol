use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::store::TestrunDataSelection;

#[function_component(TestrunShow)]
pub fn testrun() -> Html {
    let (selection, _dispatch) = use_store::<TestrunDataSelection>();

    match selection.testrun_data {
        Some(ref tr) => {
            let s = tr.statistics.as_ref().unwrap();
            html! {
                <article>
                        <h5>{ format!("{}", s.name)}</h5>

                        <div class="pure-g">

                        <div class="pure-u-4-5">
                            <table class="pure-table">
                                <thead>
                                <tr>
                                    <th>{"Request"}</th>
                                    <th>{"Count"}</th>
                                    <th>{"Errors"}</th>
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
                                            <HighlightedCell value={errors} warning_limit=0 error_limit=10 />
                                            <HighlightedCell value={x.min} />
                                            <HighlightedCell value={x.max} />
                                            <HighlightedCell value={x.avg} />
                                            <HighlightedCell value={x.p95} />
                                            </tr>
                                        }

                                    }).collect::<Html>()
                                }
                                </tbody>
                            </table>
                        </div>
                        <div class="pure-u-1-5">
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
                        </div>
                    </div>
                    </article>
            }
        }
        None => html!(),
    }
}

#[derive(Properties, PartialEq)]
pub struct HighlightedCellProps {
    pub value: u64,
    #[prop_or(800)]
    pub warning_limit: u64,
    #[prop_or(1200)]
    pub error_limit: u64,
}

#[function_component(HighlightedCell)]
pub fn hcell(props: &HighlightedCellProps) -> Html {
    let error_class = if props.value > props.error_limit {
        "functional-error"
    } else if props.value > props.warning_limit {
        "functional-warning"
    } else {
        "functional-ok"
    };
    html! {
        <td class={classes!(error_class)}>{ format!("{}", props.value )}</td>
    }
}
