# HLStatsScaper
Used as the data scraper for the HockeyStatsViewer. This has been moved out of that repo into it's own repo, here.

## The data scraping client

Build using:

```bash
cargo build
``` 

Then run the application instead the binary directory of the HSViewer application. If the application is started inside this directory
using cargo run -- . the data will be saved under a sub folder called assets/db.

If the application is *not* run inside that directory, create the subfolders assets/db and assets/db/gi_partials.

## Usage
From the terminal (or "prompt" in useless a¤¤ windows), type

Scraping the entire season (using only 1 thread)
```bash
hockeyscraper -y 2020 -d . -g 868
```

To scrape using multi-threading one must pass -j JOBS or --jobs JOBS. Scraping the entirety of the 20/21 season, using 12 threads
would look something like this:
```bash
hockeyscraper --dir ./scraped -y 2020 --games 868 --jobs 12
``` 

Options to pass to the command line tool. (Words within () are parameter values to be passed)
- Directory DIR, short form: -d (DIR) or long form --dir (DIR)
- Season STARTYEAR, short form: -y (STARTYEAR) or long form --season (STARTYEAR)
- Games in season GAMES, short form: -g (GAMES) or long form --games (GAMES)
- Jobs JOBS, short form: -j (JOBS) or long form --jobs (JOBS)
- Scrape until date (DATE), short form: -e (DATE) or long form --end (DATE) (in the format yyyymmdd)
- Run (simple) benchmark test of game results scraping, from 1..NUMCPUS threads, ITERATIONS times, short form: -r (ITERATIONS) or long form: --test_gr (ITERATIONS)
- Run (simple) benchmark test of game info scraping, from 1..NUMCPUS threads, ITERATIONS times, short form: -i (ITERATIONS) or long form: --test_gi (ITERATIONS)

If the user passes either -h or --help, the tool will only display help message, regardless of what other options are passed.