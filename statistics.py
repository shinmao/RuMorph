import sys, os

sys_root = "/home/RuMorph/"
base_path = "/home/RuMorph/crates/source/"
unsafe_pattern = "This function contains unsafe block"
b1_pattern = "BrokenLayoutChecker::analyze("
b2_pattern = "UninitExposureChecker::analyze("
b3_pattern = "BrokenBitPatternsChecker::analyze("
cast_ptr = "cast::ptr-ptr"
transmute_ptr = "transmute::ptr-ptr"
cast_found = "find the bug with behavior_flag: CAST"
transmute_found = "find the bug with behavior_flag: TRANSMUTE"
gen_pattern = "generic type conversion"

def report_analyzer(filename, crate):
    record = ""
    record_list = list()
    b1_function_captured = False
    unsafe_captured = False
    unsafe_cnt = 0
    cast_cnt = 0
    cast_captured = False
    transmute_cnt = 0
    transmute_captured = False
    if os.path.exists(filename):
        with open(filename, 'r', encoding='utf-8') as f:
            for l_num, line in enumerate(f):
                if unsafe_pattern in line:
                    print(unsafe_pattern)
                if b1_pattern in line:
                    print(b1_pattern)
                    b1_function_captured = True
                if b2_pattern in line:
                    print(b2_pattern)
                if b3_pattern in line:
                    print(b3_pattern)
        f.close()
    return record_list

def tester(path):
    cast_cnt = 0
    transmute_cnt = 0
    os.chdir( os.path.join(base_path, path) )
    '''
    subprocess.call(test_cmd, shell=True)
    subprocess.call(nonopt_cmd, shell=True)
    print(f"tester log for {path} has been created")
    '''
    if os.path.exists("report.txt"):
        with open("report.txt", "r", encoding="utf-8") as f:
            for l_num, line in enumerate(f):
                if cast_ptr in line:
                    cast_cnt += 1
                if transmute_ptr in line:
                    transmute_cnt += 1
        f.close()
    return cast_cnt, transmute_cnt

if __name__ == '__main__':
    crate_list = list()
    with open('final.txt', 'r') as metadata:
        for line in metadata:
            splitted = line.split(',')
            crate = splitted[0]
            crate_list.append(crate)
    metadata.close()
    cast_sum = 0
    transmute_sum = 0
    for crate_name in crate_list:
        if os.path.exists( os.path.join(base_path, crate_name) ):
            cast_cnt, transmute_cnt = tester( crate_name )
            print(cast_cnt)
            print(transmute_cnt)
            cast_sum += cast_cnt
            transmute_sum += transmute_cnt
    print("cast sum: ", cast_sum)
    print("transmute sum: ", transmute_sum)
    '''report_analyzer( "report.txt" )'''
