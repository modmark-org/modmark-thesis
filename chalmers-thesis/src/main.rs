use serde_json::{from_str, json, Value};
use std::{
    env,
    fmt::Write,
    io::{self, Read},
};

enum Error {
    MissingKey,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some(action) = args.get(0) else {
        eprintln!("No action was provided.");
        return;
    };

    match action.as_str() {
        "manifest" => print!("{}", &manifest()),
        "transform" => {
            let from = args.get(1).unwrap();
            let format = args.get(2).unwrap();

            if "latex" != format {
                eprintln!("Output format not supported");
                return;
            }

            match transform(from) {
                Ok(output) => print!("{output}"),
                Err(error) => handle_error(error),
            }
        }
        other => eprintln!("Invalid action '{other}'"),
    }
}

fn handle_error(error: Error) {
    match error {
        Error::MissingKey => eprintln!("Missing citation key."),
    }
}

fn transform(from: &str) -> Result<String, Error> {
    let input: Value = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        from_str(&buffer).unwrap()
    };

    match from {
        "cite" => transform_cite(input),
        "__document" => transform_document(input),
        "__heading" => transform_heading(input),
        _ => panic!("element not supported"),
    }
}

fn transform_cite(input: Value) -> Result<String, Error> {
    let key = input["data"].as_str().unwrap();
    let postnote = input["arguments"]["postnote"].as_str().unwrap();

    // You are required to provide a key
    if key.is_empty() {
        return Err(Error::MissingKey);
    }

    let args = if postnote.is_empty() {
        String::new()
    } else {
        format!("[{postnote}]")
    };

    let citation = format!("\\cite{args}{{{key}}}");
    let json = json!([{"name": "raw", "data": citation}]);

    Ok(serde_json::to_string(&json).unwrap())
}

fn transform_document(doc: Value) -> Result<String, Error> {
    let class = "\\documentclass[12pt,a4paper,twoside,openright]{report}\n";
    let prelude = "[textfile] tex/prelude.tex";
    let begin = "\\begin{document}\n";
    let end = "\n\\end{document}";

    let mut result = String::new();

    result.push('[');
    write!(result, "{},", json!({"name": "raw", "data": class})).unwrap();
    write!(
        result,
        "{},",
        json!({"name": "inline_content", "data": prelude})
    )
    .unwrap();
    write!(result, "{},", json!({"name": "raw", "data": begin})).unwrap();

    if let Value::Array(children) = &doc["children"] {
        for child in children {
            result.push_str(&serde_json::to_string(child).unwrap());
            result.push(',');
        }
    }

    write!(result, "{}", json!({"name": "raw", "data": end})).unwrap();
    result.push(']');

    Ok(result)
}

fn transform_heading(heading: Value) -> Result<String, Error> {
    let mut result = String::new();
    result.push('[');

    let level = {
        let Value::String(s) = &heading["arguments"]["level"] else {
        panic!();
    };
        s.parse::<u8>().unwrap()
    };

    if level == 1 {
        write!(result, r#"{{"name": "raw", "data": "\n\\chapter{{"}},"#,).unwrap();
    } else {
        let adjusted_level = level - 1;
        if adjusted_level > 3 {
            eprintln!("Latex only supports headings up to level 3");
        }
        let subs = "sub".repeat((adjusted_level - 1) as usize);
        write!(
            result,
            r#"{{"name": "raw", "data": "\n\\{subs}section{{"}},"#,
        )
        .unwrap();
    };

    if let Value::Array(children) = &heading["children"] {
        for child in children {
            result.push_str(&serde_json::to_string(child).unwrap());
            result.push(',');
        }
    }
    write!(result, r#"{{"name": "raw", "data": "}}\n"}}"#,).unwrap();
    result.push(']');

    Ok(result)
}

fn manifest() -> String {
    serde_json::to_string(&json!(
        {
            "version": "0.1",
            "name": "chalmers-thesis",
            "description": "A port of the Bachelor's thesis template from Chalmers University of Technology.",
            "transforms": [
                {
                    "from": "cite",
                    "to": ["latex"],
                    "arguments": [
                        {
                            "name": "postnote",
                            "description": "A note at the end of the citation, such as a page number",
                            "default": ""
                        },
                    ],
                },
                {
                    "from": "__document",
                    "to": ["latex"],
                    "arguments": [],
                },
                {
                    "from": "__heading",
                    "to": ["latex"],
                    "arguments": [
                        {
                            "name": "level",
                            "description": "The level of the heading",
                            "default": "1"
                        }
                    ],
                },

            ]
        }
    ))
    .unwrap()
}
