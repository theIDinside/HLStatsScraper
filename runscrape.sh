#!/bin/bash
rm test -rf
mkdir test

cargo.exe build --release 
cp target/release/hockeyscraper.exe ./hockeyscraper

./hockeyscraper -d test -y 2020 -g 868 -e 20210204 --jobs 12
