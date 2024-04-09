cur_max = 0
count = 0
summ = 0

with open('lines.txt', 'r') as input:
    for line, data in enumerate(input):
        data = data.rstrip()
        if data.isnumeric():
            if int(data) > cur_max:
                cur_max = int(data)
            count += 1
            summ += int(data)
input.close()

print("max", cur_max)
print("sum", summ)
print("cnt", count)
