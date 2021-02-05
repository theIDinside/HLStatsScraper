#!/bin/bash
rm test -rf
mkdir test

cargo.exe build --release 
cp target/release/hockeyscraper.exe ./hockeyscraper

# if one wants to scrape game results up until a specific date,
# one must pass "-e DATE" (format of DATE: yyyymmdd)
# -g defines the length of the season in games, which has varied when there are lockouts
# or during the pandemic. Otherwise the season is 1271 games (it will increase when they add Seattle
# in the expansion draft)

./hockeyscraper -d ../scraped -y 2020 -g 868 --jobs 12
