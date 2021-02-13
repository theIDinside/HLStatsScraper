#!/bin/bash

cargo.exe build --release 
cp target/release/hockeyscraper.exe ./hockeyscraper

./hockeyscraper -d test -y 2020 -g 868 --jobs 12
