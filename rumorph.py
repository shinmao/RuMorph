import os
import subprocess
import re

def find_rust_packages_and_run_detector(p):
        # Print the path to the package
        print(f"Found package: {p}")

        # Change the working directory to the package directory
        os.chdir(p)

        # Execute 'cargo mirai' command
        try:
            result = subprocess.run(['cargo', 'rumorph'], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            print(f"Output of cargo rumorph for {p}:\n{result.stdout.decode('utf-8')}")
        except subprocess.CalledProcessError as e:
            print(f"Error running cargo rumorph for {p}:\n{e.stderr.decode('utf-8')}")


if __name__ == "__main__":
    path = list()
    with open('./cargotree.txt', encoding="utf-8") as f:
        for line in f:
            c = line.rstrip()
            matches = re.findall(r'\(/home/certik/([^)]+)\)', c)
            for m in matches:
                p = "/home/certik/" + m
                if p not in path:
                    print(p)
                    path.append(p)
    f.close()
    for p in path:
        find_rust_packages_and_run_detector(p)
