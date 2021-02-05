#!/bin/bash
rm tests -rf
mkdir tests

cargo.exe build --release 
cp target/release/hockeyscraper.exe ./hockeyscraper

# Run benchmark for game result scraping, with 1..8 threads
./hockeyscraper --test_gi 1 -d benchmark -y 2020 -g 868 -e 20210204
# Run benchmark for game info scraping, with 1..8 threads (a heavier job, includes navigating through much more HTML data)
./hockeyscraper --test_gr 5 -d benchmark -y 2020 -g 868 -e 20210204
