# HLStatsScaper
Used as the data scraper for the HockeyStatsViewer. This has been moved out of that repo into it's own repo, here.

# The data scraping client

Build using 

```bash
cargo build
``` 

Then run the application instead the binary directory of the HSViewer application. If the application is started inside this directory
using cargo run -- . the data will be saved under a sub folder called assets/db.

If the application is *not* run inside that directory, create the subfolders assets/db and assets/db/gi_partials.
