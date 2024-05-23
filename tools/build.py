import subprocess
import json
import sys

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

def help():
    ...

functions = {
    "build" : build,
    "help" : help,
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