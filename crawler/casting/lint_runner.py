#!/usr/bin/python3.8
import os, tarfile
from multiprocessing import Pool
import subprocess
from glob import glob

tar_path = glob(os.path.join("/home/RuMorph/crates/", "*.tar.gz"))
src_path = os.listdir("/home/RuMorph/crates/source")
base_path = "/home/RuMorph/crates/source/"
lint_cmd = "cargo +nightly-2023-06-02 clippy -- -Wclippy::ptr_as_ptr > ./cast.log 2>&1"
lint_log = glob("/home/RuMorph/crates/source/*/cast.log")
fail_log_path = "/home/RuMorph/crawler/casting/fail.log"

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
                code_loc = line[ line.find("-->") + 4: ]
                record += code_loc
                CAPTURED = False
                record_list.append(record)
                record = ""
            if line.startswith('warning: Here is cast('):
                try:
                    ty_info_warn = line[22:-3].split("=>")
                    # rhs of =>
                    rhs = ty_info_warn[1].split(">")
                    mut_change = rhs[0]
                    layout = ""
                    if len(rhs) == 3:
                        layout = rhs[1] + ">" + rhs[2]
                    else:
                        layout = rhs[1]
                    # lhs of =>
                    lhs = ty_info_warn[0].split(">")
                    caller = lhs[0]
                    from_ty = lhs[1]
                    to_ty = lhs[2]
                    record += crate + "," + caller + "," + from_ty + ">" + to_ty + "," + mut_change + "," + layout + ","
                    CAPTURED = True
                except IndexError as e:
                    print(f"IndexError occurs in {path}")
                    with open(fail_log_path, 'a+') as f2:
                        f2.write(path + '\n')
                    f2.close()
                    f.close()
                    return []
    f.close()
    print(f"done for {crate}")
    return record_list

# single file writer
def writer(record):
    with open('./casting_statistics.txt', 'a+') as output:
        for r in record:
            output.write(r)

# run lint on the crates which fail the test again
def fail_log_analyzer():
    with open( fail_log_path, 'r', encoding='utf-8' ) as f:
        for l_num, line in enumerate(f):
            writer(analyzer(line.strip() + 'cast.log'))
    f.close()

# execute clippy lint in each crate source
def linter(path):
    os.chdir( os.path.join(base_path, path) )
    subprocess.call(lint_cmd, shell=True)
    print(f"cast log for {path} has been created!")

# remove all existing log files
def existing_log_rm():
    for log in lint_log:
        if os.path.exists(log):
            os.remove(log)

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
    '''
    existing_log_rm()
    with Pool(processes=20) as pool:
        pool.map(linter, src_path)
    pool.close()
    pool.join()
    '''
    for log in lint_log:
        writer(analyzer(log))
