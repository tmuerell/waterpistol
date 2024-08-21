use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    sync::Arc,
};

use gloo::file::File;
use gloo_net::http::Request;
use log::info;
use models::{config::AppConfig, RunTestParam, UploadTestsuite};
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys, HtmlInputElement};
use yew::prelude::*;

#[function_component(Uploader)]
pub fn testrun_starter() -> Html {
    let file_input = use_node_ref();
    let upload_button = use_node_ref();

    let onsubmit = {
        let file_input = file_input.clone();
        let upload_button = upload_button.clone();
    
            Callback::from(move |ev: SubmitEvent| {
            ev.prevent_default();

            let input: HtmlInputElement = file_input.cast::<HtmlInputElement>().unwrap();
            let files = input.files().unwrap();
            web_sys::console::log_1(&"lala".into());
            web_sys::console::log_1(&files.clone().into());

            let mut files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);

            let upload_button = upload_button.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let button = upload_button.cast::<web_sys::HtmlButtonElement>().unwrap();

                button.set_text_content(Some("Uploading..."));

                if let Some(f) = files.nth(0) {
                    let file_name = f.name();
                    let mime_type = f.raw_mime_type();
                    let data = gloo_file::futures::read_as_bytes(&f).await.unwrap();

                    let body = UploadTestsuite {
                        file_name,
                        mime_type,
                        data
                    };
                    let _ = Request::post("/api/testsuites/upload")
                        .json(&body)
                        .unwrap()
                        .send()
                        .await
                        .unwrap();
                }
                button.set_text_content(Some("Upload done"));

            });
        })
    };

    html! {
        <article>
            <h3>{"Uploader"}</h3>
            <form {onsubmit} class="pure-form pure-form-aligned">
                <div class="pure-control-group">
                    <label for="file-upload">{"Archive"}</label>
                    <input
                    id="file-upload"
                    ref={file_input}
                    type="file"
                    accept="application/gzip"
                />
                </div>
                <div class="pure-controls">
                    <button type="submit" ref={upload_button} class="pure-button pure-button-primary">{ "Upload archive" }</button>
                </div>
            </form>
        </article>
    }
}
