NAME = SiCaS-Whitepaper

all: $(NAME).pdf

$(NAME).pdf: $(NAME).tex ../references.bib
	pdflatex -interaction=nonstopmode $(NAME).tex
	bibtex $(NAME)
	pdflatex -interaction=nonstopmode $(NAME).tex
	pdflatex -interaction=nonstopmode $(NAME).tex

clean:
	rm -f *.aux *.bbl *.blg *.log *.out *.toc *.pdf

.PHONY: all clean
