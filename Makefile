.PHONY: help build

build:
	python3 ./tools/build.py build

help:
	python3 ./tools/build.py help

commit:
	git add . && git commit && git push