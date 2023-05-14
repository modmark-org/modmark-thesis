#![recursion_limit = "256"]

use serde_json::{from_str, json, Value};
use std::{
    env,
    io::{self, Read},
};

mod html_document;
mod latex_document;
use html_document::transform_document_html;
use latex_document::transform_document_latex;

enum Error {
    MissingKey,
    ConsumedInput,
    HeadingLevel(u8),
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
            let from = &args[1];
            let to = &args[2];

            match transform(from, to) {
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
        Error::HeadingLevel(level) => {
            eprintln!("Invalid heading level '{level}'.")
        }
        Error::ConsumedInput => eprintln!("This module should not consume any input."),
    }
}

fn transform(from: &str, to: &str) -> Result<String, Error> {
    let input: Value = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        from_str(&buffer).unwrap()
    };

    match from {
        "cite" => transform_cite(input, to),
        "__document" => transform_document(input, to),
        "__heading" => transform_heading(input, to),
        "tex" => transform_latex_command("TeX", input, to),
        "latex" => transform_latex_command("LaTeX", input, to),
        "Tex" => transform_latex_command("TeX", input, to),
        "Latex" => transform_latex_command("LaTeX", input, to),
        "note" => transform_note(input, to),
        "note-label" => transform_note_label(input, to),
        "label" => transform_label(input, to),
        "reference" => transform_reference(input, to),
        "fancy-image" => transform_fancy_image(input, to),
        "fancy-table" => transform_fancy_table(input, to),
        "fancy-big-table" => transform_fancy_big_table(input, to),
        "element-number" => transform_element_number(input, to),
        _ => panic!("element not supported"),
    }
}

fn transform_fancy_image(input: Value, to: &str) -> Result<String, Error> {
    let data = input["data"].as_str().unwrap();

    let alt = input["arguments"]["alt"].as_str().unwrap();
    let width = input["arguments"]["width"].as_f64().unwrap().to_string();
    let mut caption = input["arguments"]["caption"].as_str().unwrap().to_string();
    let cap_align = input["arguments"]["caption-alignment"].as_str().unwrap();
    let label = input["arguments"]["label"].as_str().unwrap();
    let embed = input["arguments"]["embed"].as_str().unwrap();

    // making use of the fact that caption becomes inline-content in [image]
    if to == "html" {
        caption = format!("**Figure [element-number]({label}):** {caption}");
    }

    let module_invoc = format!(
        "[image \"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\"](((\n{}\n)))",
        alt, caption, label, width, embed, cap_align, data,
    );

    let label_entry = format!("label/{}", label);
    let json = json!([
        {"name": "list-push", "arguments": {"name": "structure"}, "data": "fig"},
        {"name": "list-push", "arguments": {"name": "structure"}, "data": label_entry},
        {"name": "block_content", "data": module_invoc},
    ]);

    Ok(serde_json::to_string(&json).unwrap())
}

fn transform_fancy_table(input: Value, to: &str) -> Result<String, Error> {
    let data = input["data"].as_str().unwrap();

    let mut caption = input["arguments"]["caption"].as_str().unwrap().to_string();
    let label = input["arguments"]["label"].as_str().unwrap();
    let header = input["arguments"]["header"].as_str().unwrap();
    let alignment = input["arguments"]["alignment"].as_str().unwrap();
    let borders = input["arguments"]["borders"].as_str().unwrap();
    let delimiter = input["arguments"]["delimiter"].as_str().unwrap();
    let strip = input["arguments"]["strip_whitespace"].as_str().unwrap();

    // making use of the fact that caption becomes inline-content in [table]
    if to == "html" {
        caption = format!("**Table [element-number]({label}):** {caption}");
    }

    let module_invoc = format!(
        "[table \"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\"](((\n{}\n)))",
        caption, label, header, alignment, borders, delimiter, strip, data,
    );

    let label_entry = format!("label/{}", label);
    let json = json!([
        {"name": "list-push", "arguments": {"name": "structure"}, "data": "tab"},
        {"name": "list-push", "arguments": {"name": "structure"}, "data": label_entry},
        {"name": "block_content", "data": module_invoc},
    ]);

    Ok(serde_json::to_string(&json).unwrap())
}

fn transform_fancy_big_table(input: Value, to: &str) -> Result<String, Error> {
    let data = input["data"].as_str().unwrap();

    let mut caption = input["arguments"]["caption"].as_str().unwrap().to_string();
    let label = input["arguments"]["label"].as_str().unwrap();
    let alignment = input["arguments"]["alignment"].as_str().unwrap();

    let borders = input["arguments"]["borders"].as_str().unwrap();
    let col_delimiter = input["arguments"]["column-delimiter"].as_str().unwrap();

    let row_delimiter = input["arguments"]["row-delimiter"]
        .as_str()
        .unwrap()
        .to_string();

    if to == "html" {
        caption = format!("**Table [element-number]({label}):** {caption}");
    }

    let module_invoc = format!(
        "[big-table \"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\"](((\n{}\n)))",
        caption, label, alignment, borders, col_delimiter, row_delimiter, data,
    );

    let label_entry = format!("label/{}", label);
    let json = json!([
        {"name": "list-push", "arguments": {"name": "structure"}, "data": "tab"},
        {"name": "list-push", "arguments": {"name": "structure"}, "data": label_entry},
        {"name": "block_content", "data": module_invoc},
    ]);

    Ok(serde_json::to_string(&json).unwrap())
}

fn transform_label(input: Value, to: &str) -> Result<String, Error> {
    let label = input["data"].as_str().unwrap();

    let json = match to {
        "html" => {
            let escaped_label = label.replace('"', "%22");
            let label_tag = format!(r#"<span id="{escaped_label}">"#);
            let label_entry = format!("label/{}", label);
            let output = format!("{label_tag}</span>");
            json!([
                {"name": "list-push", "arguments": {"name": "structure"}, "data": label_entry},
                output
            ])
        }
        "latex" => {
            let escaped_label = label.replace('"', "%22");

            let output = format!(r#"\label{{{}}}"#, escaped_label);
            json!([output])
        }
        _ => {
            json!([])
        }
    };

    Ok(serde_json::to_string(&json).unwrap())
}

fn transform_heading(heading: Value, to: &str) -> Result<String, Error> {
    let mut list: Vec<Value> = vec![];
    let level = {
        let Value::String(s) = &heading["arguments"]["level"] else {
            panic!();
        };
        s.parse::<u8>().unwrap()
    };

    match to {
        "latex" => {
            let command = match level {
                1 => "\\chapter{",
                2 => "\\section{",
                3 => "\\subsection{",
                4 => "\\subsubsection{",
                5 => "\\paragraph{",
                6 => "\\subparagraph{",
                _ => return Err(Error::HeadingLevel(level)),
            };

            list.push(Value::String(command.into()));
            if let Value::Array(children) = &heading["children"] {
                for child in children {
                    list.push(child.clone());
                }
            }
            list.push(Value::String("}".into()));
        }
        "html" => {
            let key = format!("h{level}");
            list.push(
                json!({"name": "list-push", "arguments": {"name": "structure"}, "data": key}),
            );

            list.push(Value::String(format!("<h{level}>")));
            if let Value::Array(children) = &heading["children"] {
                for child in children {
                    list.push(child.clone());
                }
            }
            list.push(Value::String(format!("</h{level}>")));
        }
        _ => {}
    }

    Ok(serde_json::to_string(&Value::Array(list)).unwrap())
}

fn transform_reference(input: Value, to: &str) -> Result<String, Error> {
    match to {
        "html" => {
            let label = input["data"].as_str().unwrap();
            let mut escaped_label = label.replace('"', "%22");
            escaped_label.insert(0, '#');

            let label_tag = format!(r#"<a href="{escaped_label}">"#);
            let elem_num_invoc = format!("[element-number]({label})");

            let json = json!([
                {"name": "raw", "data": label_tag},
                {"name": "inline_content", "data": elem_num_invoc},
                {"name": "raw", "data":  "</a>"},
            ]);

            Ok(serde_json::to_string(&json).unwrap())
        }
        "latex" => {
            let label = input["data"].as_str().unwrap();
            let escaped_label = label.replace('"', "%22");

            let label_tag = format!(r#"\ref{{{}}}"#, escaped_label);

            let json = json!([
                {"name": "raw", "data": label_tag},
            ]);

            Ok(serde_json::to_string(&json).unwrap())
        }
        other => {
            eprintln!("Cannot convert ref to {other}");
            Ok(String::from("[]"))
        }
    }
}

fn transform_element_number(input: Value, _to: &str) -> Result<String, Error> {
    let label = input["data"].as_str().unwrap();
    let mut fig_count = 0;
    let mut tab_count = 0;
    let mut sec_counts = vec![0; 5];
    let mut prev = "h";
    let mut number = String::new();

    let structure: Vec<String> = {
        let var = env::var("structure").unwrap_or("[]".to_string());
        from_str(&var).unwrap()
    };

    for item in &structure {
        match item.as_str() {
            "fig" => {
                fig_count += 1;
                prev = "fig";
            }
            "tab" => {
                tab_count += 1;
                prev = "tab";
            }
            "h1" | "h2" | "h3" | "h4" | "h5" => {
                let level = item[1..].parse::<usize>().unwrap();
                for i in level..sec_counts.len() {
                    sec_counts[i] = 0;
                }
                sec_counts[level - 1] += 1;
                if item == "h1" {
                    fig_count = 0;
                    tab_count = 0;
                }

                prev = "h";
            }
            _ => {
                if Some(label) == item.strip_prefix("label/") {
                    number = match prev {
                        "fig" => format!("{}.{}", sec_counts[0], fig_count),
                        "tab" => format!("{}.{}", sec_counts[0], tab_count),
                        "h" => sec_counts
                            .iter()
                            .filter(|&&c| c != 0)
                            .map(|c| c.to_string())
                            .collect::<Vec<String>>()
                            .join("."),
                        _ => String::new(),
                    };
                    break;
                }
            }
        }
    }

    let json = json!(number);
    Ok(format!("[{json}]"))
}

fn transform_note(input: Value, to: &str) -> Result<String, Error> {
    let result = match to {
        // It's very easy when using latex, just add a \footnote{...}
        "latex" => {
            let mut result = Vec::new();
            result.push(json!(r"\footnote{"));
            result.push(json!({
                "name": "inline_content",
                "data": input["data"],
                "args": {}
            }));
            result.push(json!("}"));
            result
        }
        // When outputing html we store every note in the "notes" variable
        // tied to a randomized id and also create a new [note-label] that
        // reads from that list once it's done.
        // NOTE: a simpler alternate approach would of been
        // just reading from the list while we are pushing to it instead of using the [note-label] as a proxy,
        // but that is currently not supported by ModMark.
        "html" => {
            let mut result = Vec::new();

            let id = rand::random::<u64>();
            let payload = json!({
                "id": id,
                "note": input["data"]
            });

            result.push(json!({
                "name": "list-push",
                "arguments": {"name": "notes"},
                "data": serde_json::to_string(&payload).unwrap()
            }));

            result.push(json!({
                "name": "note-label",
                "arguments": {"id": id},
                "data": "",
            }));
            result
        }
        _ => unreachable!("unsupported format"),
    };
    Ok(serde_json::to_string(&result).unwrap())
}

fn transform_note_label(input: Value, to: &str) -> Result<String, Error> {
    let Value::Number(id) = &input["arguments"]["id"] else {
        unreachable!()
    };
    let id = id.as_u64().unwrap();

    // Find the correct note
    let number = 1 + DocSettings::get_notes()
        .into_iter()
        .enumerate()
        .find(|(_, data)| {
            if let Value::Number(note_id) = &data["id"] {
                note_id.as_u64().unwrap() == id
            } else {
                false
            }
        })
        .map(|(index, _)| index)
        .unwrap();

    match to {
        "html" => {
            let result = json!([format!(
                r##"<a id="note-backlink:{id}"></a><a href="#note:{id}"><sup>{number}</sup></a>"##
            ),]);

            Ok(serde_json::to_string(&result).unwrap())
        }

        _ => unreachable!("unsupported format"),
    }
}

/// Transform latex macros like \LaTeX and \TeX
/// and fallback to just rendering plain text if using another
/// output format
fn transform_latex_command(command: &str, input: Value, to: &str) -> Result<String, Error> {
    if let Value::String(s) = &input["data"] {
        // ensure that the input is empty
        if !s.is_empty() {
            return Err(Error::ConsumedInput);
        }

        Ok(serde_json::to_string(&match to {
            "html" => json!([command]),
            "latex" => json!([format!(r"\{command}{{}}")]),
            _ => unreachable!("Unsupported format"),
        })
        .unwrap())
    } else {
        Err(Error::ConsumedInput)
    }
}

fn transform_cite(input: Value, to: &str) -> Result<String, Error> {
    let key = input["data"].as_str().unwrap();
    let postnote = input["arguments"]["postnote"].as_str().unwrap();

    // You are required to provide a key
    if key.is_empty() {
        return Err(Error::MissingKey);
    }

    if to == "html" {
        let html =
            format!("<span style=\"background: #0000000d; font-style: italic; border-radius: 1rem; padding: 0.2rem 0.5rem 0.2rem 0.5rem; font-size: 80%;\">{key} {postnote}</span>");

        return Ok(serde_json::to_string(&json!([
            {
                "name": "raw",
                "data": html
            }
        ]))
        .unwrap());
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

struct DocSettings {
    /// Name of authors
    authors: Vec<String>,
    /// Title of thesis
    title: Option<String>,
    /// Subtitle of thesis
    subtitle: Option<String>,
    /// Department the authors belong to
    department: Option<String>,
    /// Name of the supervisor
    supervisor: Option<String>,
    /// The supervisor's department
    supervisor_department: Option<String>,
    /// Name of the (groups) examiner
    examiner: Option<String>,
    /// Dartment of the (groups) examiner
    examiner_department: Option<String>,
    /// Name of course examiner(s)
    course_examiner: Option<String>,
    /// Department of the course examiner
    course_examiner_department: Option<String>,
    /// Path to the cover art
    cover_art: Option<String>,
    /// A description of the cover art
    cover_art_description: Option<String>,
    /// The abstract text
    abstract_content: Option<String>,
    /// Sammandrag (swedish abstract)
    sammandrag: Option<String>,
    /// Keywords that are mentioned in the abstract
    keywords: Option<String>,
    /// The acknowledgement text
    acknowledgements_content: Option<String>,
    /// Used on the titlepage
    /// For instance, "Computer Science and Engineering".
    subject: Option<String>,
    /// Path to a .bib file
    sources: Option<String>,
}

impl DocSettings {
    fn from_env() -> Self {
        let authors = Self::read_authors();
        if authors.is_empty() {
            eprintln!("The list 'authors' was empty.");
        }

        Self {
            authors,
            title: Self::read_const("title"),
            subtitle: Self::read_const("subtitle"),
            department: Self::read_const("department"),
            supervisor: Self::read_const("supervisor"),
            supervisor_department: Self::read_const("supervisor_department"),
            examiner: Self::read_const("examiner"),
            examiner_department: Self::read_const("examiner_department"),
            cover_art: Self::read_const("cover_art"),
            cover_art_description: Self::read_const("cover_art_description"),
            abstract_content: Self::read_const("abstract"),
            sammandrag: Self::read_const("sammandrag"),
            acknowledgements_content: Self::read_const("acknowledgements"),
            sources: Self::read_const("sources"),
            course_examiner: Self::read_const("course_examiner"),
            course_examiner_department: Self::read_const("course_examiner_department"),
            subject: Self::read_const("subject"),
            keywords: Self::read_const("keywords"),
        }
    }

    /// Get a constant from a environment variable and log a warning if it's missing
    fn read_const(name: &str) -> Option<String> {
        let value = env::var(name).ok();

        if value.is_none() {
            eprintln!("Missing constant '{name}'.");
        }

        value
    }

    /// Get a list of all authors
    fn read_authors() -> Vec<String> {
        let Ok(variable) = env::var("authors") else {
                return Vec::new();
            };

        let Ok(Value::Array(array)) = serde_json::from_str(&variable) else {
                unreachable!("authors is of the type list");
        };

        array
            .into_iter()
            .map(|s| {
                if let Value::String(s) = s {
                    s
                } else {
                    unreachable!("ModMark lists always contain strings")
                }
            })
            .collect::<Vec<String>>()
    }

    fn get_title(&self) -> String {
        self.title
            .to_owned()
            .unwrap_or_else(|| "Missing title".to_string())
    }

    fn get_notes() -> Vec<Value> {
        let var = env::var("notes").unwrap_or_else(|_| "[]".to_string());
        let array: Value = serde_json::from_str(&var).unwrap();

        array
            .as_array()
            .unwrap()
            .into_iter()
            .map(|note| serde_json::from_str(note.as_str().unwrap()).unwrap())
            .collect()
    }
}

fn transform_document(input: Value, to: &str) -> Result<String, Error> {
    match to {
        "html" => transform_document_html(input),
        "latex" => transform_document_latex(input),
        _ => unreachable!(),
    }
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
                    "to": ["latex", "html"],
                    "arguments": [
                        {
                            "name": "postnote",
                            "description": "A note at the end of the citation, such as a page number",
                            "default": ""
                        },
                    ],
                },
                {
                    "from": "note",
                    "to": ["latex", "html"],
                    "description": "Add a note.",
                    "arguments": [],
                    "variables": {
                        "notes": {"type": "list", "access": "push"}
                    },
                },
                {
                    "from": "note-label",
                    "to": ["html"],
                    "description": "Do not use this module. It is generated when using [note].",
                    "arguments": [
                        {"name": "id", "description": "The id of the note", "type": "u64"}
                    ],
                    "variables": {
                        "notes": {"type": "list", "access": "read"}
                    },
                },
                {
                    "from": "__document",
                    "to": ["latex", "html"],
                    "arguments": [],
                    "variables": {
                        "imports": {"type": "set", "access": "read"},
                        "authors": {"type": "list", "access": "read"},
                        "title": {"type": "const", "access": "read"},
                        "subtitle": {"type": "const", "access": "read"},
                        "department": {"type": "const", "access": "read"},
                        "supervisor": {"type": "const", "access": "read"},
                        "supervisor_department": {"type": "const", "access": "read"},
                        "examiner": {"type": "const", "access": "read"},
                        "examiner_department": {"type": "const", "access": "read"},
                        "course_examiner": {"type": "const", "access": "read"},
                        "course_examiner_department": {"type": "const", "access": "read"},
                        "cover_art": {"type": "const", "access": "read"},
                        "cover_art_description": {"type": "const", "access": "read"},
                        "abstract": {"type": "const", "access": "read"},
                        "acknowledgements": {"type": "const", "access": "read"},
                        "language": {"type": "const", "access": "read"},
                        "sources": {"type": "const", "access": "read"},
                        "subject": {"type": "const", "access": "read"},
                        "keywords": {"type": "const", "access": "read"},
                        "sammandrag": {"type": "const", "access": "read"},
                        "notes": {"type": "list", "access": "read"},
                    },
                    "type": "parent"
                },
                {
                    "from": "__heading",
                    "to": ["latex", "html"],
                    "arguments": [
                        {
                            "name": "level",
                            "description": "The level of the heading",
                            "default": "1"
                        }
                    ],
                    "variables": {
                        "structure": {"type": "list", "access": "push"},
                    },
                    "type": "parent"
                },
                {
                    "from": "latex",
                    "to": ["latex", "html"],
                    "arguments": [],
                },
                {
                    "from": "tex",
                    "to": ["latex", "html"],
                    "arguments": [],
                },
                {
                    "from": "Latex",
                    "to": ["latex", "html"],
                    "arguments": [],
                },
                {
                    "from": "Tex",
                    "to": ["latex", "html"],
                    "arguments": [],
                },
                {
                    "from": "fancy-image",
                    "to": ["html", "latex"],
                    "type": "multiline-module",
                    "arguments": [
                        {"name": "alt", "default": "", "description": "Alternative text for the image"},
                        {
                            "name": "caption",
                            "default": "",
                            "description": "The caption for the image."
                        },
                        {
                            "name": "label",
                            "default": "",
                            "description": "The label to use for the image, to be able to refer to it from the document."
                        },
                        {
                            "name": "width",
                            "default": 1.0,
                            "type": "f64",
                            "description":
                                "\
                                The width of the image resulting image. \
                                For LaTeX this is ratio to the document's text area width. \
                                For HTML this is ratio to the width of the surrounding figure tag (created automatically).\
                                "
                        },
                        {
                            "name": "embed", "default": "false", "type": ["true", "false"], "description": "Decides if the provided image should be embedded in the HTML document."
                        },
                        {
                            "name": "caption-alignment", "default": "center", "type": ["left", "center", "right"], "description": "The alignment of the image caption."
                        },
                    ],
                    "variables": {
                        "structure": {"type": "list", "access": "push"},
                        "imports": {"type": "set", "access": "add"}
                    }
                },
                {
                    "from": "fancy-table",
                    "to": ["html", "latex"],
                    "arguments": [
                        {"name": "caption", "default": "", "description": "The caption for the table"},
                        {"name": "label", "default":"", "description": "The label to use for the table, to be able to refer to it from the document"},
                        {"name": "header", "default": "none", "type": ["none", "bold"], "description": "Style to apply to heading, none/bold"},
                        {"name": "alignment", "default": "left", "description": "Horizontal alignment in cells, left/center/right or l/c/r for each column"},
                        {"name": "borders", "default": "all", "type": ["all", "horizontal", "vertical", "outer", "none"], "description": "Which borders to draw"},
                        {"name": "delimiter", "default": "|", "description": "The delimiter between cells"},
                        {"name": "strip_whitespace", "default": "true", "type": ["true", "false"], "description": "true/false to strip/don't strip whitespace in cells"}
                    ],
                    "unknown-content": true,
                    "description": "Makes a table. Use one row for each row in the table, and separate the columns by the delimiter (default = |)"
                },
                {
                    "from": "fancy-big-table",
                    "to": ["html", "latex"],
                    "arguments": [
                        {"name": "caption", "default": "", "description": "The caption for the table"},
                        {"name": "label", "default":"", "description": "The label to use for the table, to be able to refer to it from the document"},
                        {"name": "alignment", "default": "left", "description": "Horizontal alignment in cells, left/center/right or l/c/r for each column"},
                        {"name": "borders", "default": "all", "type": ["all", "horizontal", "vertical", "outer", "none"], "description": "Which borders to draw"},
                        {"name": "column-delimiter", "default": "[next-column]", "description": "The delimiter between columns"},
                        {"name": "row-delimiter", "default": "[next-row]", "description": "The delimiter between rows"},
                    ],
                    "unknown-content": true,
                    "description": "Large variant of the table, which accepts block content. Write the content of each cell on multiple lines, and use column-delimiter between cells on the same row. Then, use row-delimiter between rows."
                },
                {
                    "from": "label",
                    "to": ["html", "latex"],
                    "arguments": [],
                    "variables": {
                        "structure": {"type": "list", "access": "push"},
                    }
                },
                {
                    "from": "reference",
                    "to": ["html", "latex"],
                    "arguments": [],
                    "variables": {
                        "structure": {"type": "list", "access": "read"}
                    }
                },
                {
                    "from": "element-number",
                    "to": ["any"],
                    "arguments": [],
                    "variables": {
                        "structure": {"type": "list", "access": "read"}
                    }
                },
            ]
        }
    ))
    .unwrap()
}
