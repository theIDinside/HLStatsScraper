#!/bin/bash
# The FFI library exists in the same workspace folder (one level up, then in project folder is stattrends, therefore ../stattrends)

# For testing of library. We scrape to this directory, then copy to workspace folder of the Application
asset_dir="../statsdb/statsdb_bin"

cargo.exe build --release 
cp target/release/hockeyscraper.exe ./hockeyscraper

./hockeyscraper -d $asset_dir -y 2020 -g 868 --jobs 12

# all projects have hsv_workspace as rootfolder
cp $asset_dir/assets ../builds -r
