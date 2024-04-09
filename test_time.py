import time, subprocess, os

os.chdir('/home/RuMorph/crates/source/ag-0.19.2')
start_time = time.time()
subprocess.call('cargo rumorph', shell=True)
print(time.time() - start_time)
