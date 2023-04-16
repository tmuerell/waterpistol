use std::collections::HashSet;

use gloo_utils::document;
use log::info;
use plotly::common::Title;
use plotly::layout::Axis;
use plotly::{Layout, Plot, Scatter};
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::store::CompareSelection;

#[derive(Default, PartialEq)]
enum CriteriaSelection {
    #[default]
    Avg,
    Max,
    Min,
    P95,
}

struct CompareData {
    request: String,
    data: Vec<Option<u64>>,
}

#[function_component(CompareView)]
pub fn compare() -> Html {
    let (selection, _dispatch) = use_store::<CompareSelection>();
    let criteria = use_state(|| CriteriaSelection::default());

    let selection_for_plot = selection.clone();
    let criteria_for_plot = criteria.clone();

    // Plotter
    let p = yew_hooks::use_async::<_, _, ()>({
        async move {
            let id = "plot-div";
            let mut plot = Plot::new();

            match selection_for_plot.testrun_data {
                Some(ref tr) => {
                    let mut tr = tr.clone();
                    tr.sort_by(|a, b| a.datum.cmp(&b.datum));

                    let requests: HashSet<String> = tr
                        .iter()
                        .flat_map(|x| {
                            x.statistics
                                .as_ref()
                                .unwrap()
                                .request_stats
                                .iter()
                                .map(|d| d.name.clone())
                        })
                        .collect();
                    let mut requests: Vec<String> = requests.into_iter().collect();
                    requests.sort();

                    let x_axis: Vec<String> = tr
                        .iter()
                        .map(|x| x.datum.map(|d| d.to_rfc3339()).unwrap_or("".to_string()))
                        .collect();

                    for r in requests {
                        let data: Vec<Option<u64>> = tr
                            .iter()
                            .map(|td| {
                                td.statistics
                                    .as_ref()
                                    .unwrap()
                                    .request_stats
                                    .iter()
                                    .find(|r2| &r2.name == &r)
                                    .map(|r| match *criteria_for_plot {
                                        CriteriaSelection::Avg => r.avg,
                                        CriteriaSelection::Max => r.max,
                                        CriteriaSelection::Min => r.min,
                                        CriteriaSelection::P95 => r.p95,
                                    })
                            })
                            .collect();

                        let trace = Scatter::new(x_axis.clone(), data).name(&r);
                        plot.add_trace(trace);
                    }

                    let layout = Layout::new()
                        .title(Title::new("Speed graph"))
                        .x_axis(Axis::new().title(Title::new("Time")))
                        .y_axis(Axis::new().title(Title::new("Milliseconds")));
                    plot.set_layout(layout);

                    info!("Trying to plot");

                    if document().get_element_by_id(id).is_some() {
                        info!("Really plotting!");
                        plotly::bindings::new_plot(id, &plot).await;
                    }
                }
                None => (),
            }
            Ok(())
        }
    });

    let selection_dep = selection.clone();
    let criteria_dep = criteria.clone();
    use_effect_with_deps(
        move |_| {
            p.run();
            || ()
        },
        (selection_dep, criteria_dep),
    );

    let canvas_ref = use_node_ref();
    /*

    let canvas_ref2 = canvas_ref.clone();
    use_effect(move || {
        if let Some(ca) = canvas_ref2.cast::<HtmlCanvasElement>() {
            draw(ca, 12.0, 1.0);
        }
    });
     */

    match selection.testrun_data {
        Some(ref tr) => {
            let mut tr = tr.clone();
            tr.sort_by(|a, b| a.datum.cmp(&b.datum));

            let requests: HashSet<String> = tr
                .iter()
                .flat_map(|x| {
                    x.statistics
                        .as_ref()
                        .unwrap()
                        .request_stats
                        .iter()
                        .map(|d| d.name.clone())
                })
                .collect();
            let mut requests: Vec<String> = requests.into_iter().collect();
            requests.sort();

            let data: Vec<_> = requests
                .iter()
                .map(|r| CompareData {
                    request: r.to_owned(),
                    data: tr
                        .iter()
                        .map(|td| {
                            td.statistics
                                .as_ref()
                                .unwrap()
                                .request_stats
                                .iter()
                                .find(|r2| &r2.name == r)
                                .map(|r| match *criteria {
                                    CriteriaSelection::Avg => r.avg,
                                    CriteriaSelection::Max => r.max,
                                    CriteriaSelection::Min => r.min,
                                    CriteriaSelection::P95 => r.p95,
                                })
                        })
                        .collect(),
                })
                .collect();

            let comparision_method = match *criteria {
                CriteriaSelection::Avg => "Average",
                CriteriaSelection::Min => "Minimum",
                CriteriaSelection::Max => "Maximum",
                CriteriaSelection::P95 => "Perc95",
            };

            html! {
                <article>
                    <div style="float: right;">

                    <p>
                        <a onclick={ let criteria = criteria.clone();
                            Callback::from(move |_| {
                            criteria.set(CriteriaSelection::Avg)
                        })}>{ "[Avg]" }</a>
                        <a onclick={ let criteria = criteria.clone();
                            Callback::from(move |_| {
                            criteria.set(CriteriaSelection::Min)
                        })}>{ "[Min]" }</a>
                        <a onclick={ let criteria = criteria.clone();
                            Callback::from(move |_| {
                            criteria.set(CriteriaSelection::Max)
                        })}>{ "[Max]" }</a>
                        <a onclick={ let criteria = criteria.clone();
                            Callback::from(move |_| {
                            criteria.set(CriteriaSelection::P95)
                        })}>{ "[P95]" }</a>                    </p>

                    </div>
                    <h3>{ format!("Compare {}", comparision_method) }</h3>
                        <canvas ref={canvas_ref}></canvas>
                        <div id="plot-div"></div>

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

/*
pub fn draw_plotters(canvas: HtmlCanvasElement, pitch: f64, yaw: f64) -> Result<(), Box<dyn Error>> {
    let area = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();
    area.fill(&WHITE)?;

    let x_axis = (-3.0..3.0).step(0.1);
    let z_axis = (-3.0..3.0).step(0.1);

    let mut chart =
        ChartBuilder::on(&area).build_cartesian_3d(x_axis.clone(), -3.0..3.0, z_axis.clone())?;

    chart.with_projection(|mut pb| {
        pb.yaw = yaw;
        pb.pitch = pitch;
        pb.scale = 0.7;
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    chart.draw_series(
        SurfaceSeries::xoz(x_axis.values(), z_axis.values(), |x: f64, z: f64| {
            (x * x + z * z).cos()
        })
        .style(&BLUE.mix(0.2)),
    )?;

    chart.draw_series(LineSeries::new(
        (-100..100)
            .map(|y| y as f64 / 40.0)
            .map(|y| ((y * 10.0).sin(), y, (y * 10.0).cos())),
        &BLACK,
    ))?;

    Ok(())
}
 */
