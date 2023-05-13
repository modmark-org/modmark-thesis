use crate::{DocSettings, Error};
use serde_json::{json, Value};
use std::{collections::HashSet, env, fmt::Write};

pub(crate) fn transform_document_latex(input: Value) -> Result<String, Error> {
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

    content.append(&mut create_abstract(&settings));

    content.append(&mut create_acknowledgements(&settings));

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
    if let Value::Array(children) = &input["children"] {
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
        \includegraphics[width=\linewidth]{{{cover_art}}}
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
    {{\Large {subtitle}}}\\[0.3cm]"#
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
    content.push_str(r" \setlength{\parskip}{0.5cm}");
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
	
    {department}
	\textsc{{Chalmers University of Technology}} \\
	\textsc{{University of Gothenburg}} \\
	Gothenburg, Sweden \the\year \\
\end{{center}}",
        department = format!("{}\\", &settings.department.to_owned().unwrap_or_default()),
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
            .map(|name| name.replace(" ", "~").to_uppercase())
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
            "Supervisor (handledare): {supervisor}, {department}\\\\"
        )
        .unwrap();
    }

    if let Some(course_examiner) = &settings.course_examiner {
        let department = settings
            .course_examiner_department
            .to_owned()
            .unwrap_or_default();

        writeln!(
            &mut content,
            "Examiners: {course_examiner}, {department}\\\\"
        )
        .unwrap();
    }

    if let Some(examiner) = &settings.examiner {
        let department = settings.examiner_department.to_owned().unwrap_or_default();

        writeln!(
            &mut content,
            "Graded by teacher (rättande lärare): {examiner}, {department}\\\\"
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

fn create_acknowledgements(settings: &DocSettings) -> Vec<Value> {
    let mut content = Vec::new();
    let Some(acknowledgements) = settings.acknowledgements_content.to_owned() else {
        return content;
    };

    content.push(Value::String(
        "\\newpage\n\\thispagestyle{plain}\n\n\\section*{Acknowledgements}".to_string(),
    ));

    content.push(json!({"name": "block_content", "data": acknowledgements, "args": {}}));

    content.push(Value::String(
        r"\vspace{1.5cm}
\hfill
\begin{flushright}"
            .to_string(),
    ));

    content.push(Value::String(
        settings
            .authors
            .iter()
            .map(|name| name.replace(" ", "~"))
            .collect::<Vec<_>>()
            .join(", "),
    ));

    content.push(Value::String(
        r"
Gothenburg, \monthname \space \the\year
\end{flushright}
    
\newpage				% Create empty back of side
\thispagestyle{empty}
\mbox{}"
            .to_string(),
    ));

    content
}

fn create_abstract(settings: &DocSettings) -> Vec<Value> {
    let mut content = Vec::new();

    let Some(abstract_content) = &settings.abstract_content else {
        // Just return an empty string if there is no abstract defined
        return content;
    };
    content.push(Value::String("\\newpage\n".to_string()));
    content.push(Value::String(format!(
        "\\textbf{{{}}}",
        settings.get_title()
    )));
    content.push(Value::String("\\\\ \n".to_string()));

    if let Some(subtitle) = &settings.subtitle {
        content.push(Value::String(format!("{subtitle}\\\\ \n")));
    }

    // authors
    content.push(Value::String(
        "\n\n\\parbox{0.8\\textwidth}{\n\\begin{flushleft}".to_string(),
    ));
    content.push(Value::String(
        settings
            .authors
            .iter()
            .map(|name| name.replace(" ", "~"))
            .collect::<Vec<_>>()
            .join(", "),
    ));

    content.push(Value::String("\n\\end{flushleft}\n} \\\\ \n\n".to_string()));

    if let Some(department) = &settings.department {
        content.push(Value::String(format!(r"{department} \\")));
    }

    content.push(Value::String(
        r"
Chalmers University of Technology and University of Gothenburg\setlength{\parskip}{0.5cm}

\thispagestyle{plain}
\setlength{\parskip}{0pt plus 1.0pt}
\section*{Abstract}
"
        .to_string(),
    ));

    // Add the contents of the abstract
    content.push(json!({"name": "block_content", "data": abstract_content, "args": {}}));

    // If there is an abstract in swedish add that too
    if let Some(sammanfattning) = &settings.sammanfattning {
        content.push(Value::String("\\section*{Sammanfattning}".to_string()));
        content.push(json!({"name": "block_content", "data": sammanfattning, "args": {}}));
    }

    if let Some(keywords) = &settings.keywords {
        content.push(Value::String(format!(
            r"
    \vfill
    Keywords: {keywords}"
        )));
    }

    // Finally, add a empty back page
    content.push(Value::String(
        r"
\newpage
\thispagestyle{empty}
\mbox{}"
            .to_string(),
    ));

    content
}
