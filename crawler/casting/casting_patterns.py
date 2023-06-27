#!/usr/bin/python3
from pandas import *
import collections

def conv_txt_to_csv(input_path, output_path):
    data = read_csv(input_path)
    data.to_csv(output_path, index=None)

if __name__ == '__main__':
    txt_path = "./casting_statistics.txt"
    csv_path = "./casting_statistics.csv"
    conv_txt_to_csv(txt_path, csv_path)
    data = read_csv(csv_path)
    conv = data['conv'].tolist()
    col = collections.Counter(conv)
    with open("casting_patterns.txt", 'a+') as f:
        for pt_cnt in col.most_common():
            st = ''
            f.write(st.join(map(str, pt_cnt)) + '\n')
    f.close()
