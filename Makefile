.PHONY: build run commit install_deps

build:
	cargo bootimage

run:
    # qemu-system-x86_64 -drive format=raw,file=target/x86_64-baremetal/debug/bootimage-primoria.bin
	cargo run

clean:
	rm -Rf ./target

commit:
	git add .
	git commit
	git push

install_deps:
	rustup component add rust-src --toolchain nightly
	cargo install bootimage
	rustup component add llvm-tools-preview
