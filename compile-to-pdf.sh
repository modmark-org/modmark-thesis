./compile-to-tex.sh
mkdir build
mv thesis.tex build/
cd build
ln -s ../figures
ln -s ../include
pdflatex -synctex=1 -interaction=nonstopmode --shell-escape thesis.tex
cd ..
mv build/thesis.pdf .
