# Guide
The latest database dump is provided in the address https://crates.io/data-access.  

Download the `db-dump.tar.gz` file and put in the root of `crawler`. Run `cargo run` then we can get the list of crates that are most downloaded.  

In order to download the packages following the list, we need to run `crates_downloader.py` in the root.  

Note: the expression `:=` is not supported until python 3.8, make sure to run it with `python3.8`.

# File structure
* `crates_list.txt`: The most downloaded packages as of 2023-06-16.
* `crates_listII.txt`: The most downloaded packages as of 2024-03-18.

# Reference
* [How Rust Search Extension Indexes Top 20k crates](https://rustmagazine.org/issue-3/how-rse-index-top-20k-crates/)
