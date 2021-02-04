#!/bin/bash
rm test -rf
mkdir test
cargo.exe run -- -d test -y 2020 -g 868 -e 20210204 --jobs 6
# Run it twice. The first time, there will be no GameInfos scraped, and as it stands, 
# the user must run the tool twice, once for GameInfos and once for GameResults. This is bound 
# to change
cargo.exe run -- -d test -y 2020 -g 868 -e 20210204 --jobs 6
