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
            let cb = Closure::wrap(Box::new(move |_ev: web_sys::ProgressEvent| {
                log!("Loaded file");
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

    let (template, set_template) = create_signal("".to_owned());

    let rows = move || {
        let csv = csv();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let header = reader.headers().ok().cloned();
        reader
            .records()
            .into_iter()
            .filter_map(|l| l.ok())
            .map(move |line| {
                let mut row = serde_json::Map::default();

                for (k, v) in header.as_ref().unwrap().iter().zip(line.iter()) {
                    row.insert(k.into(), v.into());
                }

                serde_json::Value::Object(row)
            })
            .collect::<Vec<_>>()
    };

    let md = move || {
        let mut reg = handlebars::Handlebars::new();
        let template = template.get();
        reg.register_template_string("template", template.as_str())
            .unwrap();
        let rows = rows();
        rows.into_iter()
            .map(|row| reg.render("template", &row).unwrap())
            .map(|row| view! {<pre>{row}</pre>})
            .collect_view()
    };

    let update_templates = move |ev| {
        set_template.update(move |x| *x = event_target_value(&ev));
    };

    view! {
        <p>CSV file: {csv}</p>
        <input type="file" accept=".csv" placeholder="csv file" node_ref=csv_input/>
        <textarea value=template on:change=update_templates></textarea>
        {md}
    }
}
