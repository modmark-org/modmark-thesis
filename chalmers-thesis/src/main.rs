use serde_json::{from_str, json, Value};
use std::{
    collections::HashSet,
    env,
    fmt::Write,
    io::{self, Read},
};

enum Error {
    MissingKey,
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
        _ => panic!("element not supported"),
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
            acknowledgements_content: Self::read_const("acknowledgements"),
            sources: Self::read_const("sources"),
            course_examiner: Self::read_const("course_examiner"),
            course_examiner_department: Self::read_const("course_examiner_department"),
            subject: Self::read_const("subject"),
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
}

fn transform_document(doc: Value, _to: &str) -> Result<String, Error> {
    let settings = DocSettings::from_env();

    let mut content: Vec<Value> = Vec::new();

    // Get a hashset of all imports that are needed
    let imports = {
        let mut imports = get_template_imports();
        // add all imports coming from other packages
        if let Ok(other_imports) = env::var("imports") {
            if let Value::Array(array) = serde_json::from_str(&other_imports).unwrap() {
                imports.extend(array.into_iter().map(|value| {
                    if let Value::String(s) = value {
                        s
                    } else {
                        unreachable!()
                    }
                }))
            }
        }
        imports
    };

    // Declare the document class
    content.push(Value::String(
        "\\documentclass[12pt,a4paper,twoside,openright]{report}\n".into(),
    ));

    // Add all the imports seperated by newlines
    content.push(Value::String(
        imports.into_iter().collect::<Vec<_>>().join("\n"),
    ));

    // Add settings (helper macros and such)
    content.push(Value::String(include_str!("settings.tex").into()));

    // Import the bibliography
    if let Some(sources) = &settings.sources {
        content.push(Value::String(format!("\\addbibresource{{{sources}}}\n")));
    }

    // Start the document
    content.push(Value::String("\\begin{document}\n".into()));

    // Add the cover, title and imprint pages
    content.push(Value::String(create_coverpage(&settings)));
    content.push(Value::String(create_titlepage(&settings)));
    content.push(Value::String(create_imprint_page(&settings)));

    // FIXME: add abstract and acknowledgements

    // table of contents and start of main content
    content.push(Value::String(
        r"
\newpage
\tableofcontents

\cleardoublepage
\setcounter{page}{1}
\pagenumbering{arabic}
\setlength{\parskip}{0.5 \baselineskip plus 2pt}
"
        .into(),
    ));

    // Add the main content of the document
    if let Value::Array(children) = &doc["children"] {
        for child in children {
            content.push(child.clone());
        }
    };

    if settings.sources.is_some() {
        content.push(Value::String(format!(
            r"
\cleardoublepage    
\addcontentsline{{toc}}{{chapter}}{{Bibliography}}
\printbibliography
"
        )));
    }

    content.push(Value::String("\\end{document}".into()));

    Ok(serde_json::to_string(&content).unwrap())
}

fn transform_heading(heading: Value, _to: &str) -> Result<String, Error> {
    let mut list: Vec<Value> = vec![];
    let level = {
        let Value::String(s) = &heading["arguments"]["level"] else {
            panic!();
        };
        s.parse::<u8>().unwrap()
    };

    let (command, closing) = match level {
        1 => ("\\chapter{", "}"),
        2 => ("\\section{", "}"),
        3 => ("\\subsection{", "}"),
        4 => ("\\subsubsection{", "}"),
        5 => ("\\paragraph{\\underline{", "}}"),
        6 => ("\\subparagraph{", "}"),
        _ => return Err(Error::HeadingLevel(level)),
    };

    list.push(Value::String(command.into()));

    if let Value::Array(children) = &heading["children"] {
        for child in children {
            list.push(child.clone());
        }
    }
    list.push(Value::String(closing.into()));

    Ok(serde_json::to_string(&Value::Array(list)).unwrap())
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
                    "from": "__document",
                    "to": ["latex"],
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
                        "abstract_content": {"type": "const", "access": "read"},
                        "acknowledgements_content": {"type": "const", "access": "read"},
                        "language": {"type": "const", "access": "read"},
                        "sources": {"type": "const", "access": "read"},
                        "subject": {"type": "const", "access": "read"}
                    }
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

fn get_template_imports() -> HashSet<String> {
    ["\\usepackage[top=3cm,bottom=3cm,inner=3cm,outer=3cm]{geometry}".to_string(),
    "\\usepackage{parskip}".to_string(),
    "\\usepackage{textcomp}".to_string(),
    "\\usepackage{lmodern}".to_string(),
    "\\usepackage{helvet}".to_string(),
    "\\usepackage[T1]{fontenc}".to_string(),
    "\\usepackage[utf8]{inputenc}".to_string(),
    "\\usepackage[english]{babel}".to_string(),
    "\\usepackage{graphicx}".to_string(),
    "\\usepackage{float}".to_string(),
    "\\usepackage[hidelinks]{hyperref}".to_string(),
    "\\usepackage[normalem]{ulem}".to_string(),
    "\\usepackage{svg}".to_string(),
    "\\usepackage{adjustbox}".to_string(),
    "\\usepackage{datetime}".to_string(),
    "\\usepackage{csquotes}".to_string(),
    "\\usepackage[backend=biber,style=ieee,urldate=comp,block=ragged]{biblatex}".to_string(),
    "\\usepackage{titlesec}".to_string(),
    "\\usepackage{eso-pic}".to_string(),
    "\\usepackage[labelfont=bf,textfont=normal,justification=justified,singlelinecheck=false]{caption}".to_string(),
    "\\usepackage{fancyhdr}".to_string(), 
    "\\usepackage{xpatch}".to_string()
    ].into()
}

fn create_coverpage(settings: &DocSettings) -> String {
    let mut content = String::new();
    content.push_str(
        r#"
\pagenumbering{roman}	
\begin{titlepage}
    \newgeometry{top=3cm, bottom=3cm, left=2.25 cm, right=2.25cm}	% Temporarily change margins
    \AddToShipoutPicture*{\put(55,150){\includesvg{coverpage.svg}}}
    \addtolength{\voffset}{2cm}"#,
    );

    if let Some(cover_art) = &settings.cover_art {
        write!(
            &mut content,
            r#"
    
    % Cover art		
    \begin{{figure}}[H]
        \centering
        \vspace{{1cm}} 
        \includegraphics[width=0.9\linewidth]{{{cover_art}}}
    \end{{figure}}"#,
        )
        .unwrap();
    }

    content.push_str(
        r#"
    
    % Cover text
    \mbox{}
    \vfill
    \renewcommand{\familydefault}{\sfdefault} \normalfont % Set cover page font"#,
    );

    // Main title
    let title = settings.get_title();

    writeln!(
        &mut content,
        r#"
    \textbf{{\Huge {title}}}"#
    )
    .unwrap();

    // If there is a subtitle add that too
    if let Some(subtitle) = &settings.subtitle {
        writeln!(
            &mut content,
            r#"
    {{\Large {subtitle}}}\\[0.5cm]"#
        )
        .unwrap();
    }

    // Subject line
    content.push_str(
        r"
    Bachelor's thesis",
    );

    if let Some(subject) = &settings.subject {
        write!(&mut content, r#" in {subject}"#).unwrap();
    }
    content.push_str(r" \setlength{\parskip}{1cm}");
    content.push_str("\n\n");

    // Names of authors
    // {\Large AUTHOR's NAME} \setlength{\parskip}{2.9cm}\\[1ex] ...
    content.push_str(
        &settings
            .authors
            .iter()
            .map(|name| format!(r#" {{\Large {name}}} \setlength{{\parskip}}{{2.9cm}}"#))
            .collect::<Vec<_>>()
            .join("\\\\[1ex]\n"),
    );

    content.push_str("\n\n");

    // End of the cover page (as well as a blank page on the back of the same paper)
    if let Some(department) = &settings.department {
        writeln!(&mut content, r"{department}\\").unwrap();
    }
    content.push_str(
        r"\textsc{Chalmers University of Technology} \\
    \textsc{University of Gothenburg} \\
    Gothenburg, Sweden \the\year
    
    \renewcommand{\familydefault}{\rmdefault} \normalfont % Reset standard font
\end{titlepage}
    
\newpage
\restoregeometry
\thispagestyle{empty}
\mbox{}",
    );

    content
}

fn create_titlepage(settings: &DocSettings) -> String {
    let mut content = String::new();

    // Start a new page and some text at the top
    content.push_str(
        r"
\newpage
\thispagestyle{empty}
\begin{center}

\textsc{\large Bachelor's thesis \the\year}\\[4cm]
",
    );

    // Main title
    let title = settings.get_title();
    writeln!(
        &mut content,
        r#"
    \textbf{{\Large {title}}} \\[1cm]"#
    )
    .unwrap();

    // If there is a subtitle add that too
    if let Some(subtitle) = &settings.subtitle {
        writeln!(
            &mut content,
            r#"
    {{\large {subtitle}}}\\[1cm]"#
        )
        .unwrap();
    }

    // List the name of all authors again too
    content.push_str(
        &settings
            .authors
            .iter()
            .map(|name| format!(r#" {{\large {name}}}"#))
            .collect::<Vec<_>>()
            .join("\\\\[1ex]\n"),
    );

    content.push_str("  \\vfill");

    content.push_str(&format!(
        r"
\begin{{figure}}[H]
    \centering
    \includesvg[width=0.25\pdfpagewidth]{{guandchalmerslogo}}
\end{{figure}} \vspace{{5mm}}	
	
    {department}\\
	\textsc{{Chalmers University of Technology}} \\
	\textsc{{University of Gothenburg}} \\
	Gothenburg, Sweden \the\year \\
\end{{center}}",
        department = &settings.department.to_owned().unwrap_or_default()
    ));

    content
}

fn create_imprint_page(settings: &DocSettings) -> String {
    let mut content = String::new();

    content.push_str(
        r"
\newpage
\thispagestyle{plain}
\vspace*{4.5cm}",
    );

    // Add title and subtitle
    let title = settings.get_title();
    writeln!(&mut content, r"\textbf{{{title}}}\\").unwrap();

    if let Some(subtitle) = &settings.subtitle {
        writeln!(&mut content, r"{subtitle}\\").unwrap();
    }

    // Add authors
    // NAME1~FAMILYNAME1 \setlength{\parskip}{1cm}
    content.push_str(
        &settings
            .authors
            .iter()
            .map(|name| {
                format!(
                    r#"{name} \setlength{{\parskip}}{{1cm}}"#,
                    name = name.replace(" ", "~")
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );

    content.push_str("\n\n");

    // Add copyright
    // \copyright ~ NAME1~FAMILYNAME1, ... \the\year. \setlength{\parskip}{1cm}
    content.push_str(r"\copyright ~ ");
    content.push_str(
        &settings
            .authors
            .iter()
            .map(|name| name.replace(" ", "~"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    content.push_str(r" \the\year. \setlength{\parskip}{1cm}");

    content.push_str("\n\n");

    if let Some(supervisor) = &settings.supervisor {
        let department = settings
            .supervisor_department
            .to_owned()
            .unwrap_or_default();

        writeln!(
            &mut content,
            "Supervisor (handledare): {supervisor}, {department}"
        )
        .unwrap();
    }

    if let Some(course_examiner) = &settings.course_examiner {
        let department = settings
            .course_examiner_department
            .to_owned()
            .unwrap_or_default();

        writeln!(&mut content, "{course_examiner}, {department}").unwrap();
    }

    if let Some(examiner) = &settings.examiner {
        let department = settings.examiner_department.to_owned().unwrap_or_default();

        writeln!(
            &mut content,
            "Graded by teacher (rättande lärare): {examiner}, {department}"
        )
        .unwrap();
    }

    content.push_str("\n\n");
    content.push_str(r"\setlength{\parskip}{1cm}");
    content.push_str("\n\n");

    content.push_str(r"Bachelor's Thesis \the\year\\");

    if let Some(department) = &settings.department {
        content.push_str(department);
    }

    content.push_str(
        r"
Chalmers University of Technology and University of Gothenburg\\
SE-412 96 Gothenburg\\
Telephone +46 31 772 1000 \setlength{\parskip}{0.5cm}

\vfill

",
    );

    if let Some(cover_description) = &settings.cover_art_description {
        writeln!(&mut content, "Cover: {cover_description}").unwrap();
    }

    content.push_str("\n\n");

    content.push_str(
        r"
\includesvg[width=5cm]{modmark}\\
Typeset using \LaTeX \\
Gothenburg, Sweden \the\year",
    );

    content
}
