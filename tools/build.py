import sys

def build():
    ...

def help():
    ...

functions = {
    "build" : build,
    "help" : help,
}

def main():
    if len(sys.argv) < 2:
        print("No arguments specified, please use the Makefile")
        return
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