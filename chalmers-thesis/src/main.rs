use serde_json::{from_str, json, Value};
use std::{
    fmt::Write,
    env,
    io::{self, Read},
};

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

            print!("{}", transform(from));
        }
        other => eprintln!("Invalid action '{other}'"),
    }
}

fn transform(from: &str) -> String {
    let input: Value = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        from_str(&buffer).unwrap()
    };

    match from {
        "titlepage" => transform_titlepage(),
        "__document" => transform_document(input),
        _ => panic!("element not supported"),
    }
}

fn transform_titlepage() -> String {
    let json = json!({"name": "inline_content", "data": "[textfile] tex/titlepage.tex"}).to_string();
    format!("[{json}]")
}

fn transform_document(doc: Value) -> String {
    let class = "\\documentclass[12pt,a4paper,twoside,openright]{report}\n";
    let prelude = "[textfile] tex/prelude.tex";
    let begin = "\\begin{document}\n";
    let end = "\n\\end{document}";

    let mut result = String::new();

    result.push('[');
    write!(result, "{},", json!({"name": "raw", "data": class})).unwrap();
    write!(result, "{},", json!({"name": "inline_content", "data": prelude})).unwrap();
    write!(result, "{},", json!({"name": "raw", "data": begin})).unwrap();

    if let Value::Array(children) = &doc["children"] {
        for child in children {
            result.push_str(&serde_json::to_string(child).unwrap());
            result.push(',');
        }
    }

    write!(result, "{}", json!({"name": "raw", "data": end})).unwrap();
    result.push(']');

    result
}

fn manifest() -> String {
    serde_json::to_string(&json!(
        {
            "version": "0.1",
            "name": "chalmers-thesis",
            "description": "",
            "transforms": [
                {
                    "from": "titlepage",
                    "to": ["latex"],
                    "arguments": [],
                },
                {
                    "from": "__document",
                    "to": ["latex"],
                    "arguments": [],
                },
            ]
        }
    ))
    .unwrap()
}
