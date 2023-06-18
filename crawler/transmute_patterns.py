#!/usr/bin/python3
from pandas import *
import collections

if __name__ == '__main__':
    data = read_csv("./transmute_stats.csv")
    conv = data['conv'].tolist()
    col = collections.Counter(conv)
    with open("transmute_patterns.txt", 'a+') as f:
        for pt_cnt in col.most_common():
            st = ''
            f.write(st.join(map(str, pt_cnt)) + '\n')
    f.close()
