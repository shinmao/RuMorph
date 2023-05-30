#!/bin/bash
input="./crates_list.txt"
cnt=0
while IFS= read -r line
do
  echo "$line"
  cargo download $line >../crates/$line.gz
  ((cnt=cnt+1))
done < "$input"
echo "$cnt crates downloaded!"