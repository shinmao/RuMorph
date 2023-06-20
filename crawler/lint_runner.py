#!/usr/bin/python3.8
import os, tarfile
from multiprocessing import Pool
import subprocess
from glob import glob

tar_path = glob(os.path.join("/home/RuMorph/crates/", "*.tar.gz"))
src_path = os.listdir("/home/RuMorph/crates/source")
base_path = "/home/RuMorph/crates/source/"
lint_cmd = "cargo +nightly-2023-06-02 clippy -- -Wclippy::transmute_statistics > ./lint.log 2>&1"
lint_log = glob("/home/RuMorph/crates/source/*/lint.log")

# analyzer the lint.log in each crate source
def analyzer(path):
    CAPTURED = False
    crate = path.split("/")[-2]
    print(crate)
    record = ""
    record_list = list()
    with open(path, 'r', encoding='utf-8') as f:
        for l_num, line in enumerate(f):
            if CAPTURED:
                code_loc = line
                record += code_loc
                CAPTURED = False
                record_list.append(record)
                record = ""
            if line.startswith('warning: Here is transmute('):
                ty_info_warn = line[27:-3].split("=>")
                record += crate + "," + ty_info_warn[0] + "," + ty_info_warn[1] + ","
                CAPTURED = True
    f.close()
    print(f"done for {crate}")
    return record_list

# single file writer
def writer(record):
    with open('./transmute_statistics.txt', 'a+') as output:
        for r in record:
            output.write(r)

# check how many log files have been created
def lint_log_checker():
    print( sum(1 for c in lint_log) )

# execute clippy lint in each crate source
def linter(path):
    os.chdir( os.path.join(base_path, path) )
    subprocess.call(lint_cmd, shell=True)
    print(f"lint log for {path} has been created!")

# unzip the crate source
def unzip(path):
    target = os.path.join("/home/RuMorph/crates/source/", path.split("/")[-1].rstrip(".tar.gz"))
    if not os.path.exists(target):
        try:
            with tarfile.open(path, "r") as tf:
                tf.extractall("/home/RuMorph/crates/source")
            tf.close()
        except tarfile.ReadError:
            print(f"read error on {path}!")

if __name__ == '__main__':
    #with Pool(processes=20) as pool:
    #    pool.map(linter, src_path)
    #pool.close()
    #pool.join()
    #lint_log_checker()
    for log in lint_log:
        writer(analyzer(log))
