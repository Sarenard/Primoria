import subprocess
import json
import sys
import os

with open("./tools/config.json") as f:
    options = json.load(f)

def run_command(command: str):
    ret = subprocess.run(command, shell=True).returncode
    if ret:
        print("Oops")
        exit(1)

def run():
    run_command("qemu-system-x86_64 -drive format=raw,file=target/x86_64-baremetal/debug/bootimage-primoria.bin")

def clean():
    run_command("rm -Rf ./out/*")
    run_command("rm -Rf ./build/*")

def help():
    ...

functions = {
    # "build" : build,
    "help" : help,
    "run" : run,
    "clean" : clean,
}

# main function
def main():
    if len(sys.argv) < 2:
        print("No arguments specified, please use the Makefile")
        exit(1)
    
    arg = sys.argv[1]

    if arg in functions:
        functions[arg]()
    else:
        print("Unknown argument, please use the Makefile")
        exit(1)

if __name__ == "__main__":
    main()
else:
    print("Why are you importing that as a module? wtf")