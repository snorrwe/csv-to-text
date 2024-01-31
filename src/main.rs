use leptos::logging::{error, log};
use leptos::*;
use leptos_use::use_event_listener;
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    FileReader,
};

fn main() {
    leptos::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (csv, set_csv) = create_signal("".to_owned());
    let csv_input: NodeRef<html::Input> = create_node_ref();

    let _ = use_event_listener(csv_input, ev::change, move |ev| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);

        if let Some(file) = target.files().and_then(|f| f.get(0)) {
            let reader = FileReader::new().expect("Failed to create filereader");
            let r = reader.clone();
            let cb = Closure::wrap(Box::new(move |ev: web_sys::ProgressEvent| {
                log!("Loaded: {:?}", ev.as_string());
                log!("???? {:?}", r.result());
                match r.result() {
                    Ok(content) => {
                        let content = content.as_string().unwrap();
                        set_csv(content);
                    }
                    Err(err) => {
                        error!("Failed to read file: {err:?}");
                    }
                }
            }) as Box<dyn Fn(web_sys::ProgressEvent)>);

            // TODO: handle errors
            reader.read_as_text(&file).expect("Failed to read file");
            reader.set_onload(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }
    });

    view! {
        <p>CSV file: {csv}</p>
        <input type="file" accept=".csv" placeholder="csv file" node_ref=csv_input/>
    }
}
