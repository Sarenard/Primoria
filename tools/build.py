import subprocess
import json
import sys
import os

def run_command(command: str):
    ret = subprocess.run(command, shell=True).returncode
    if ret:
        print("Oops")
        exit(1)

def get_sysroot() -> str:
    sysroot_cmd = [
        "rustc",
        "--print",
        "sysroot",
    ]
    output = subprocess.Popen(
        sysroot_cmd, stdout=subprocess.PIPE
    ).communicate()[0]
    sysroot = output.decode('utf-8')[:-1]
    return sysroot

def generate_json(sysroot):
    json_struct = {
        "sysroot" : sysroot,
        "crates": [
            {
                "root_module" : "src/main.rs",
                "edition" : "2021",
                "deps" : [],
                "cfg" : [],
                "env" : {

                },
                "is_proc_macro" : False
            }
        ]
    }
    return json_struct

def build():
    # we make the rust-project.json file first
    sysroot = get_sysroot()
    json_struct = generate_json(sysroot)
    serialized = json.dumps(json_struct, sort_keys=True, indent=4)
    with open("rust-project.json", "w") as f:
        f.write(serialized)

    # we build the main thing
    run_command("cp -r src build")
    run_command("rustc ./build/src/main.rs -o ./out/main")

def run():
    run_command("./out/main")

def clean():
    run_command("rm -Rf ./out/*")
    run_command("rm -Rf ./build/*")

def help():
    ...

functions = {
    "build" : build,
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