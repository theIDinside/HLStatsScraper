# HLStatsScaper
Used as the data scraper for the HockeyStatsViewer. This has been moved out of that repo into it's own repo, here.

## The data scraping client

Build using 

```bash
cargo build
``` 

Then run the application instead the binary directory of the HSViewer application. If the application is started inside this directory
using cargo run -- . the data will be saved under a sub folder called assets/db.

If the application is *not* run inside that directory, create the subfolders assets/db and assets/db/gi_partials.

## Usage

From the terminal (or "prompt" in useless a¤¤ windows), type

```bash
hockeyscraper -y 2020 -d . -g 868
```

To scrape the results and game informations for the 2020 season. As the season is shortened
by covid, the option -g must be passed with 868, since that's the total of games. Otherwise
it will scrape the ordinary amount of games. -y is the parameter for the season (starting year).
-d sets the root directory where the files and data will be stored. The directory must contain
assets/db and assets/db/gi_partials
