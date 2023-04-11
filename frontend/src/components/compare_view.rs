use std::collections::HashSet;

use plotly::{Plot, Scatter};
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::store::CompareSelection;

#[derive(Default)]
enum CriteriaSelection {
    #[default]
    Avg,
    Max,
    Min,
    P95,
}

struct CompareData {
    request: String,
    data: Vec<Option<u64>>
}

#[function_component(CompareView)]
pub fn compare() -> Html {
            // Plotter
            /*
            let p = yew_hooks::use_async::<_, _, ()>({
                let id = "plot-div";
                let mut plot = Plot::new();
                let trace = Scatter::new(vec![0, 1, 2], vec![2, 1, 0]);
                plot.add_trace(trace);
        
                async move {
                    plotly::bindings::new_plot(id, &plot).await;
                    Ok(())
                }
            });
        
            
                use_effect_with_deps(move |_| {
                    p.run();
                    || ()
                }, (),
            );
             */

let (selection, _dispatch) = use_store::<CompareSelection>();
    let criteria = use_state(|| CriteriaSelection::default());

    match selection.testrun_data {
        Some(ref tr) => {

            let requests : HashSet<String> = tr.iter().flat_map(|x| x.statistics.as_ref().unwrap().request_stats.iter().map(|d| d.name.clone() )).collect();

            let data : Vec<_> = requests.iter().map(|r| {
                CompareData {
                    request: r.to_owned(),
                    data: tr.iter().map(|td| {
                        td.statistics.as_ref().unwrap().request_stats.iter().find(|r2| &r2.name == r).map(|r| {
                            match *criteria {
                                CriteriaSelection::Avg => r.avg,
                                CriteriaSelection::Max => r.max,
                                CriteriaSelection::Min => r.min,
                                CriteriaSelection::P95 => r.p95,
                            }
                        })
                    }).collect()
                }
            }).collect();

            let comparision_method = match *criteria {
                CriteriaSelection::Avg => "Average",
                CriteriaSelection::Min => "Minimum",
                CriteriaSelection::Max => "Maximum",
                CriteriaSelection::P95 => "Perc95",
            };

            html! {
                <article>
                        <h3>{ format!("Compare {}", comparision_method) }</h3>

                        <div class="pure-g">

                        <div class="pure-u-1-1">
                            <table class="pure-table">
                                <thead>
                                <tr>
                                    <th>{"Request"}</th>
                                    {
                                        tr.iter().map(|x|
                                            html!{
                                                <th>{ &x.datum.map(|d| d.format("%Y-%m-%d %H:%M").to_string() ).unwrap_or("n/a".to_string()) }</th>
                                            }
                                        ).collect::<Html>()
                                    }
                                </tr>
                                </thead>
                                <tbody>
                                {
                                    data.iter().map(|x| {
                                        html!{
                                            <tr>
                                                <td>{ &x.request }</td>
                                                {
                                                    x.data.iter().map(|x| {
                                                        match x {
                                                            Some(v) => html!(<td>{ format!("{}", v) }</td>),
                                                            None => html!(<td></td>)
                                                        }
                                                    }).collect::<Html>()
                                                }
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
