import time, subprocess, os

start_time = time.time()
with open('final_crate_list.txt', 'r') as f:
    for line in f:
        path = '/home/RuMorph/cratesII/' + line.rstrip()
        if not os.path.exists(path):
            continue
        os.chdir(path)
        subprocess.call('cargo rumorph', shell=True)
f.close()
print(time.time() - start_time)
