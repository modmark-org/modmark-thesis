# Modmark bachelor's thesis
This repo contains the thesis for the [ModMark](https://modmark.org) project, the entire thesis is written in the ModMark language itself (see the "assets" directory) and can be compiled into a LaTeX or HTML document.


## Setup
Define the env variable `MODMARK_PATH` that points to the Cargo.toml for the ModMark CLI. Something like this:
```sh
export MODMARK_PATH="/home/user/path/to/modmark/cli/Cargo.toml"
```

## Transpile to LaTeX
Run
```sh
./compile-to-tex.sh
```

This will first build the `chalmers-thesis` ModMark package and then transpile the document using the ModMark compiler.

### PDF output
Run the following commands to get a PDF from your LaTeX file. Note that depending on your system may need to install multiple TeX packages before being able to successfully compile the document.
```sh
cd out
pdflatex thesis.tex
```
Or alternativly import `thesis.tex` inside of TeXstudio (note that you have to change texstudio to use biber instead of bibtex, see "Options -> Configure TeXstudio -> Build").

## Live preview
While writing it might be easier to use a live HTML preview instead of compiling down to a pdf file. To do so, run:
```
./live-preview.sh
``` 