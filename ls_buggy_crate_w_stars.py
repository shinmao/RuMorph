import csv

buggy_crate_list = list()
with open('./report.txt', 'r') as f:
    for line in f:
        crate = line.rstrip().split(',')[0]
        if crate not in buggy_crate_list:
            buggy_crate_list.append(crate)
f.close()

sorted_crate = list()
with open('star_crate_list.csv', 'r') as f:
    csv_reader = csv.DictReader(f)
    line_cnt = 0
    for row in csv_reader:
        if line_cnt == 0:
            line_cnt += 1
            continue
        if row['crate'] in buggy_crate_list:
            sorted_crate.append(row['crate'])
        line_cnt += 1
f.close()

with open('buggy_sorted_crate.txt', 'w+') as f:
    for sc in sorted_crate:
        f.write(sc)
        f.write('\n')
f.close()
