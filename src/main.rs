#![feature(iterator_try_collect)]

use csv::StringRecord;
use leptos::logging::{error, log};
use leptos::*;
use leptos_meta::{provide_meta_context, Stylesheet};
use leptos_use::use_event_listener;
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    FileReader,
};

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    leptos::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/tailwind.css"/>

        <main class="my-10">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-10">
                <div>
                    <Help/>
                </div>
                <div>
                    <CsvConverter/>
                </div>
            </div>
        </main>
    }
}

#[component]
fn Help() -> impl IntoView {
    const EXAMPLE_CSV: &str = r#"foo,bar
1,tiggers
24,winnie"#;

    const EXAMPLE_BODY: &str = r#"# {{bar}}'s adventures"#;
    const EXAMPLE_FILENAME: &str = r#"{{filename}}-{{foo}}-{{i}}.md"#;
    const EXAMPLE_OUT: &[&str] = &["# tigger's adventures", "# winnie's adventures"];
    view! {
        <div>
            <h1 class="text-3xl mb-4">"Usage"</h1>
            <p>"Upload a csv file."</p>
            <p>
                "Add a "
                <a
                    target="_blank"
                    href="https://handlebarsjs.com/guide/expressions.html#basic-usage"
                >
                    "Handlebars template"
                </a> " on how to render the file body"
            </p>
            <p>
                "Add a "
                <a
                    target="_blank"
                    href="https://handlebarsjs.com/guide/expressions.html#basic-usage"
                >
                    "Handlebars template"
                </a> " for the filename"
            </p>
            <p>"Click 'Download'"</p>
        </div>
        <div class="my-10">
            <h2 class="text-2xl mb-4">"Example"</h2>
            <p class="text-xl">"File `example.csv`:"</p>
            <pre class="pl-4">{EXAMPLE_CSV}</pre>
            <p class="text-xl">"Template:"</p>
            <pre class="pl-4">{EXAMPLE_BODY}</pre>
            <p class="text-xl">"Filename:"</p>
            <pre class="pl-4">{EXAMPLE_FILENAME}</pre>
            <p class="text-xl">"Produces:"</p>
            <ul>
                <li>
                    <p class="text-xl">"example-1-1.md"</p>
                    <pre class="pl-4">{EXAMPLE_OUT[0]}</pre>
                </li>
                <li>
                    <p class="text-xl">"example-24-2.md"</p>
                    <pre class="pl-4">{EXAMPLE_OUT[1]}</pre>
                </li>
            </ul>
        </div>
    }
}

#[component]
fn CsvConverter() -> impl IntoView {
    let (csv, set_csv) = create_signal("".to_owned());
    let (file_name, set_file_name) = create_signal("".to_owned());
    let csv_input: NodeRef<html::Input> = create_node_ref();

    let _ = use_event_listener(csv_input, ev::change, move |ev| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);

        if let Some(file) = target.files().and_then(|f| f.get(0)) {
            set_file_name.update(|f| *f = file.name());
            let reader = FileReader::new().expect("Failed to create filereader");
            let r = reader.clone();
            let cb = Closure::wrap(Box::new(move |_ev: web_sys::ProgressEvent| {
                log!("File loading done");
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

    let (csv_columns, set_csv_columns) = create_signal(Default::default());
    let (head_rows, set_head_rows) = create_signal(Vec::default());
    let (csv_error, set_csv_error) = create_signal(None);

    create_effect(move |_| {
        log!("Parsing csv head");
        let csv = csv();
        match csv_to_json_rows(csv.as_str(), Some(5)) {
            Ok(TemplateRows { header, rows }) => {
                if let Some(header) = header.as_ref() {
                    set_csv_columns(header.clone());
                }
                set_head_rows.update(|r| *r = rows);
                set_csv_error.update(|e| {
                    e.take();
                });
            }
            Err(err) => {
                error!("Failed to parse csv: {err:#?}");
                set_csv_error.update(|e| {
                    *e = Some(view! { <p class="text-red-500">{format!("{err}")}</p> })
                });
            }
        };
    });

    let csv_headers = move || {
        csv_columns()
            .iter()
            .map(|title| {
                view! {
                    <li class="flex flex-row">
                        <pre>{title.to_owned()}</pre>
                    </li>
                }
            })
            .collect_view()
    };

    let (template, set_template) = create_signal("".to_owned());
    let (template_err, set_template_err) = create_signal(None);

    let template_reg = move || {
        let mut reg = handlebars::Handlebars::new();
        let template = template.get();
        match reg.register_template_string("template", template.as_str()) {
            Ok(_) => set_template_err.update(|x| {
                x.take();
            }),
            Err(err) => {
                set_template_err.update(|x| {
                    error!("Failed to parse template: {err:#?}");
                    *x = Some(view! { <pre class="red">{format!("{}", err)}</pre> }.into_view());
                });
            }
        }

        reg
    };

    let preview = move || {
        log!("Rendering preview");
        let reg = template_reg();
        let rows = head_rows();
        rows.into_iter()
            .map(|row| reg.render("template", &row))
            .map(|row| match row {
                Ok(row) => view! { <pre class="gap-y-5">{row}</pre> }.into_view(),
                Err(err) => {
                    view! { <p class="red">"Failed to render template: " {format!("{}", err.reason())}</p> }
                        .into_view()
                }
            })
            .collect_view()
    };

    let update_template = move |ev| {
        set_template.update(move |x| *x = event_target_value(&ev));
    };

    let download_element: NodeRef<html::A> = create_node_ref();

    let (filename_template, set_filename_template) =
        create_signal("{{filename}}-{{i}}.md".to_owned());

    let on_download = move |ev: leptos::ev::SubmitEvent| {
        log!("Downloading rendered files");
        ev.prevent_default();
        let mut reg = template_reg();
        if let Err(err) = reg.register_template_string("filename", &filename_template()) {
            set_template_err.update(|e| {
                *e = Some(view! { <p class="text-red-500">"Failed to parse filename template: " {format!("{err}")}</p> }.into_view())
            });
        }
        let csv = csv();
        let prefix = std::path::PathBuf::from(file_name());
        let prefix = prefix.with_extension("");
        let TemplateRows { header: _, rows } = match csv_to_json_rows(csv.as_str(), None) {
            Ok(x) => x,
            Err(err) => {
                error!("Failed to parse csv: {err:#?}");
                set_csv_error.update(|e| {
                    *e = Some(view! { <p class="text-red-500">{format!("{err}")}</p> })
                });
                return;
            }
        };
        let Some(download_element) = download_element.get() else {
            return;
        };
        for (i, mut row) in rows.into_iter().enumerate() {
            if let Ok(rendered) = reg.render("template", &row) {
                // download as file
                let payload = format!(
                    "data:text;charset=utf-8,{}",
                    urlencoding::encode(rendered.as_str())
                );
                {
                    let r = row.as_object_mut().unwrap();
                    r.entry("filename")
                        .or_insert_with(|| prefix.display().to_string().into());
                    r.entry("i").or_insert_with(|| (i + 1).into());
                }
                let name = match reg.render("filename", &row) {
                    Ok(x) => x,
                    Err(err) => {
                        error!("Failed to render filename: {err:#?}");
                        continue;
                    }
                };
                download_element.set_download(name.as_str());
                download_element.set_href(payload.as_str());
                download_element.click();
            }
        }
    };

    view! {
        <div>
            <input type="file" accept=".csv" placeholder="csv file" node_ref=csv_input/>
            <div>"Columns: " <ul class="flex flex-row gap-4 max-100">{csv_headers}</ul></div>
            <div>
                <label for="template">"Body template"</label>

                {move || template_err.get()}
                {move || csv_error.get()}
            </div>

            <textarea
                class="w-auto h-auto resize border-2 border-gray-400"
                name="template"
                value="template"
                on:change=update_template
            ></textarea>
            <div>
                <h2 class="h2">"Preview:"</h2>
                {preview}
            </div>

            <form on:submit=on_download>
                <div>
                    <label for="filename-template">"Filename: "</label>
                    <input
                        type="text"
                        class="border-2 border-gray-400"
                        value=filename_template
                        on:change=move |ev| {
                            set_filename_template.update(|f| *f = event_target_value(&ev))
                        }

                        name="filename-template"
                    />
                </div>
                <div class="gap-4">
                    <button type="submit" class="hover:cursor-pointer border-2 bg-green text-white">
                        "Download"
                    </button>
                </div>
            </form>

            <a style="display:none" node_ref=download_element></a>
        </div>
    }
}

struct TemplateRows {
    header: Option<StringRecord>,
    rows: Vec<serde_json::Value>,
}

fn csv_to_json_rows(csv: &str, limit: Option<usize>) -> Result<TemplateRows, csv::Error> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let header = reader.headers().ok().cloned();
    let h = header.clone();
    reader
        .records()
        .into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .map(move |line| {
            line.map(|line| {
                let mut row = serde_json::Map::default();

                for (k, v) in header.as_ref().unwrap().iter().zip(line.iter()) {
                    row.insert(k.into(), v.into());
                }

                serde_json::Value::Object(row)
            })
        })
        .try_collect::<Vec<_>>()
        .map(|rows| TemplateRows { header: h, rows })
}
