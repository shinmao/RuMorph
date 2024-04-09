import os, subprocess, time

path = [
    #"/home/RuMorph/crates/source/fyrox-core-0.23.0/",
    #"/home/RuMorph/crates/source/libafl-0.10.1/",
    #"/home/RuMorph/crates/source/arrow-buffer-40.0.0/",
    #"/home/RuMorph/crates/source/tract-data-0.20.5/",
    #"/home/RuMorph/crates/source/vm-memory-0.11.0/",
    "/home/RuMorph/crates/source/parquet-40.0.0/"
]

test_cmd = "cargo rumorph"
test2_cmd = "cargo rumorph -- -Zdisable-optimize"

for p in path:
    print(p)
    os.chdir(p)
    start_time = time.time()
    subprocess.call(test_cmd, shell=True)
    opt_time = time.time() - start_time
    start2_time = time.time()
    subprocess.call(test2_cmd, shell=True)
    nonopt_time = time.time() - start2_time
    with open('time_overhead.txt', 'a+') as output:
        output.write(p + ', ' + str(opt_time - nonopt_time) + '\n')
