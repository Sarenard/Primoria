.PHONY: help build run clean commit

build:
	python3 ./tools/build.py build

help:
	python3 ./tools/build.py help

run:
	python3 ./tools/build.py run

clean:
	python3 ./tools/build.py clean

commit:
	git add . && git commit && git push