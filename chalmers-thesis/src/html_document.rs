use crate::{DocSettings, Error};
use serde_json::{json, Value};
use std::env;

macro_rules! raw {
    ($expr:expr) => {
        json!({
            "name": "raw",
            "data": $expr
        })
    }
}

pub(crate) fn transform_document_html(mut input: Value) -> Result<String, Error> {
    let settings = DocSettings::from_env();

    let mut result = vec![raw!(
        r#"
<!DOCTYPE html>
<html>
<head>
<title>Document</title>
<meta charset="UTF-8">
"#
    )];
    /*
     <div class="cover">
      <img src="figures/cover.jpg" />
      <h1 class="title">
        ModMark: A Modular Markup Language
      </h1>
      <div class="subtitle">Designing and implementing a markup language utilising WebAssembly</div>
      <ul class="authors">
        <li>Eli Adelhult</li>
        <li>Gustav Bruhn</li>
        <li>Eli Adelhult</li>
        <li>Gustav Bruhn</li>
        <li>Eli Adelhult</li>
        <li>Gustav Bruhn</li>
      </ul>
    </div>
    <div class="abstract">

    </div>
    */
    // Add imports
    let mut imports = {
        let var = env::var("imports").unwrap_or("[]".to_string());
        serde_json::from_str(&var).unwrap()
    };
    result.append(&mut imports);

    result.push(raw!("<style>"));
    result.push(raw!(include_str!("template.css")));
    result.push(raw!(
        r#"
</style>
</head>
<body>
<article>
<div class="cover">
"#
    ));

    // cover
    if let Some(cover) = &settings.cover_art {
        result.push(raw!(format!(r#"<img alt="Cover photo" src="{cover}"/>"#)));
    }

    // title
    let title = settings.get_title();
    result.push(raw!(format!(r#"<h1 class="title">{title}</h1>"#)));

    if let Some(subtitle) = settings.subtitle {
        result.push(raw!(format!(r#"<div class="subtitle">{subtitle}</div>"#)));
    }

    // authors
    result.push(raw!(r#"<ul class="authors">"#));
    result.push(raw!(&settings
        .authors
        .iter()
        .map(|name| format!(r#"<li>{name}</li>"#))
        .collect::<Vec<_>>()
        .join("\n")));
    result.push(raw!("</ul>"));
    result.push(raw!("</div>"));

    let include_preamble = settings.abstract_content.is_some()
        || settings.sammanfattning.is_some()
        || settings.acknowledgements_content.is_some();

    if include_preamble {
        result.push(raw!(r#"<div class="preamble">"#));
    }

    // abstract
    if let Some(abstract_content) = &settings.abstract_content {
        result.push(raw!("<h2>Abstract</h2>"));
        result.push(json!({"name": "block_content", "data": abstract_content, "args": {}}));
    }

    // sammanfattning (swe. abstract)
    if let Some(sammanfattning) = &settings.sammanfattning {
        result.push(raw!("<h2>Sammanfattning</h2>"));
        result.push(json!({"name": "block_content", "data": sammanfattning, "args": {}}));
    }

    // acknowledgements
    if let Some(acknowledgements) = &settings.acknowledgements_content {
        result.push(raw!("<h2>Acknowledgements</h2>"));
        result.push(json!({"name": "block_content", "data": acknowledgements, "args": {}}));
    }

    if include_preamble {
        result.push(raw!("</div>"));
    }

    // content
    if let Some(children) = input.get_mut("children") {
        if let Value::Array(ref mut children) = children {
            result.append(children);
        } else {
            unreachable!("Children is not a list");
        }
    }

    result.push(raw!("</article></body></html>"));

    Ok(serde_json::to_string(&result).unwrap())
}
