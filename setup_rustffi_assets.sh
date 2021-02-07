#!/bin/bash
# The FFI library exists in the same workspace folder (one level up, then in project folder is stattrends, therefore ../stattrends)


cargo.exe build --release 
cp target/release/hockeyscraper.exe ./hockeyscraper

./hockeyscraper -d ../stattrends -y 2020 -g 868 --jobs 12

