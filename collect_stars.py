import json

star_dict = {}
with open('./star_sorted_crateII.txt', 'r') as star:
    for line in star:
        splitted = line.rstrip().split(',')
        name = splitted[0]
        star = splitted[1]
        star_dict[name] = star

#print(star_dict)

crate_dict = {}
version_dict = {}
with open('whole_crates_list.txt', 'r') as crate:
    for line in crate:
        splitted = line.rstrip().split(',')
        crate = splitted[0]
        name = splitted[2]
        crate_dict[name] = crate
        version_dict[crate] = splitted[1]

#print(crate_dict)

final_dict = {}
for key, val in star_dict.items():
    if key in crate_dict:
        print(key)
        final_dict[key] = val

final_list = sorted(final_dict.items(), key=lambda x: int(x[1]), reverse=True)
final_d = dict(final_list)

with open('final_gt100.txt', 'a+') as output:
    for (key, value) in iter(final_d.items()):
        print(key)
        print(value)
        output.write(crate_dict[key] + "-" + str( version_dict[ crate_dict[key] ] ) + "\n")
output.close()
