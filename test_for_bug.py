from multiprocessing import Pool
import subprocess
from glob import glob
import os

src_path = os.listdir("/home/RuMorph/crates/source")
base_path = "/home/RuMorph/crates/source/"
test_cmd = "cargo rumorph > report.txt 2>&1"
header1 = "Error (BrokenLayout:):"
header2 = "Error (UninitExposure:):"
header3 = "Error (BrokenBitPatterns:):"
report_path = "/home/RuMorph/report.txt"

def report_analyzer(filename, crate):
    record = ""
    record_list = list()
    CAPTURED = False
    REPORTED = False
    if os.path.exists(filename):
        with open(filename, 'r', encoding='utf-8') as f:
            for l_num, line in enumerate(f):
                if header1 in line:
                    func = line.split("`")[1]
                    record = crate + "," + func + ",bug1,"
                    CAPTURED = True
                elif header2 in line:
                    func = line.split("`")[1]
                    record = crate + "," + func + ",bug2,"
                    CAPTURED = True
                elif header3 in line:
                    func = line.split("`")[1]
                    record = crate + "," + func + ",bug3,"
                    CAPTURED = True
                elif CAPTURED == True:
                    loc = line[3:]
                    record += loc
                    CAPTURED = False
                    REPORTED = True
                elif REPORTED == True:
                    record_list.append(record)
                    record = ""
                    REPORTED = False
                else:
                    continue
        f.close()
    return record_list

def tester(path):
    os.chdir( os.path.join(base_path, path) )
    subprocess.call(test_cmd, shell=True)
    print(f"tester log for {path} has been created")

if __name__ == '__main__':
    # with Pool(processes=20) as pool:
    #    pool.map(tester, src_path)
    # pool.close()
    # pool.join()
    with open(report_path, "a+") as output:
        for src in src_path:
            path = os.path.join(base_path, src) + "/report.txt"
            for r in report_analyzer(path, src):
               output.write(r)
               print(r)
    output.close()
