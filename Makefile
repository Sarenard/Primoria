.PHONY: help build run clean commit install_deps

build:
	# python3 ./tools/build.py build # build with rustc
	# cargo +nightly build
	cargo bootimage

help:
	python3 ./tools/build.py help

run:
	python3 ./tools/build.py run

clean:
	python3 ./tools/build.py clean

commit:
	git add .
	git commit
	git push

install_deps:
	rustup component add rust-src --toolchain nightly
	cargo install bootimage
	rustup component add llvm-tools-preview
