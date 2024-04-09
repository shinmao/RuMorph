from multiprocessing import Pool
import subprocess
from glob import glob
import os, time, sys

src_path = os.listdir("/home/RuMorph/cratesII")
base_path = "/home/RuMorph/cratesII/"
test_cmd = "cargo rumorph > report.txt 2>&1"
nonopt_cmd = "cargo rumorph -- -Zdisable-optimize > report_opt.txt 2>&1"
header1 = "Error (BrokenLayout:):"
header2 = "Error (UninitExposure:):"
header3 = "Error (BrokenBitPatterns:):"
report_path = "/home/RuMorph/report.txt"

def report_analyzer(filename, crate):
    record = ""
    record_list = list()
    CAPTURED = False
    REPORTED = False
    analyzed = False
    if os.path.exists(filename):
        with open(filename, 'r', encoding='utf-8') as f:
            '''
            if "[rumorph-progress] bug not found" in f.read():
                analyzed = True
            '''
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
    comp_path = os.path.join(base_path, path)
    if not os.path.exists(comp_path):
        return
    os.chdir( comp_path )
    #test_cmd = "find . -name '*.rs' | xargs wc -l | grep 'total' | tr -dc '0-9' >> /home/RuMorph/lines.txt"
    subprocess.call(test_cmd, shell=True)
    #subprocess.call("echo '\n' >> /home/RuMorph/lines.txt", shell=True)
    #subprocess.call(nonopt_cmd, shell=True)
    print(f"tester log for {path} has been created")

if __name__ == '__main__':
    crate_list = list()
    with open('crates_listII.txt', 'r') as metadata:
        for line in metadata:
            splitted = line.split(',')
            crate = splitted[0]
            ver = splitted[1]
            crate_list.append(crate + '-' + ver)
    metadata.close()
    
    option = sys.argv[1]
    if option == "scan_bug":
        print("scan bug!!")
        start_time = time.time()
        with Pool(processes=20) as pool:
            pool.map(tester, crate_list)
        pool.close()
        pool.join()
        for c in crate_list:
            tester(c)
        
        with open("/home/RuMorph/execution.txt", "w+") as exe:
            exe.write(str(time.time() - start_time))
        exe.close()
    elif option == "scan_report":
        print("scan report!!")
        # count = 0
        with open(report_path, "a+") as output:
            for src in crate_list:
                path = os.path.join(base_path, src) + "/report.txt"
                print(path)
                for r in report_analyzer(path, src):
                    output.write(r)
                    print(r)
        output.close()
    
    '''
    for c in crate_list:
        subprocess.call("echo " + c + " >> /home/RuMorph/lines.txt", shell=True)
        tester(c + "/src")
    '''
