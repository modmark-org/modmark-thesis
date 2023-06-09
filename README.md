# Modmark bachelor's thesis
This repo contains the thesis for the [ModMark](https://modmark.org) project, the entire thesis is written in the ModMark language itself (see "main.mdm" and the "include" directory) and can be compiled into a LaTeX or HTML document.


## Setup
Define the env variable `MODMARK_PATH` that points to the root of the ModMark repo. Something like this:
```sh
export MODMARK_PATH="/home/user/path/to/modmark"
```

## Transpile to LaTeX
Run
```sh
./compile-to-tex.sh
```

This will first build the `chalmers-thesis` ModMark package and then transpile the document using the ModMark compiler.

### PDF output
The easiest way to convert the generated `.tex` file into a pdf is by installing and using TeXstudio. Remember use biber instead of bibtex,  see "Options -> Configure TeXstudio -> Build". Also, the template uses the svg package which uses inkspace to convert svgs. This means that inkscape needs to be installed and you also need to use the [`--shell-escape` option for pdflatex](https://tex.stackexchange.com/questions/99475/how-to-invoke-latex-with-the-shell-escape-flag-in-texstudio-former-texmakerx).

## Live preview
While writing it might be easier to use a live HTML preview instead of compiling down to a pdf file. To do so, run:
```
./live-preview.sh
``` 