#!/bin/bash
input="./fail.log"
while IFS= read -r line
do
	cd $line
	cargo +nightly-2023-06-02 clippy -- -Wclippy::transmute-statistics > ./lint.log 2>&1
	echo "$line log has been recreated!"
done < "$input"
