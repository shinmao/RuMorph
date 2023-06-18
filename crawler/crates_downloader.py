#!/usr/bin/python3.8
# The file helps us download the source code of 20k crates
import subprocess

def source_download(name, version, crates_cnt) -> int:
    cmd = "curl -L 'https://crates.io/api/v1/crates/{crate}/{ver}/download' > ../crates/{crate}-{ver}.tar.gz".format(crate=name, ver=version)
    subprocess.call(cmd, shell = True)
    print(f"crate: {name} downloaded!\n")
    crates_cnt += 1
    print(f"We have downloaded {crates_cnt} crates so far\n")
    return crates_cnt

if __name__ == '__main__':
    crates_cnt = 0
    path = "./crates_list.txt"
    with open(path, "r") as cl:
        while line := cl.readline():
            name, ver = [ line.rstrip().split(",")[i] for i in (0, 1) ]
            crates_cnt = source_download(name, ver, crates_cnt)
    cl.close()

