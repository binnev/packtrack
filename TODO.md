- functionality
    - [x] `config` command to allow setting a default postcode
    - `url prune` cli command to prune delivered packages 
    - more advanced url file where you can add annotations, for those urls that don't contain your postcode. YAML?
    - `raw` command that fetches raw data for a url/barcode
    - handle case where barcode in url != barcode from API
- cache the responses for each package
    - so you can show the changes over time.
        - show when a package has changed status
    - don't store a new entry if a dupe exists
    - global --cache options
        - number of responses to store per url 
        - how recent should an entry be to be reused
        - force no cache `-n` `--no-cache`
        - set cache max age in seconds `-c` `--cache-age`
    - settings for 
        - cache max hit age 
        - cache max entries per url
    - [x] also reuse recently fetched responses
    - [x] composed Tracker struct with child 
        - Tracker
        - Cache
- display
    - tui spinners when waiting for tasks
    - flag to control package display detail level (brief / detailed)
    - show how long it took to gather each package
    - show URL next to errors so we know which error belongs to which URL
    - wrap text to 80 characters
    - extract more detail from events on a channel level if possible
        - e.g. instead of `[Tue 29 Oct 08:30] UNDERWAY: PROCESSED_AT_LOCATION`, show more information about the processing, show any updates to eta, etc. 
    - option to sort packages in different ways
        - status > time > carrier 
            DELIVERED 
                Today 12:00 PostNL ... 
                Today 13:00 GLS ... 
                Today 14:00 PostNL ... 
        - status > carrier > time
            DELIVERED 
                GLS Today 13:00 ... 
                PostNL Today 12:00 ... 
                PostNL Today 14:00 ... 
        - carrier > status > time 
            POSTNL
                Today 12:00 ... 
                Today 14:00 ... 
            GLS
                Today 13:00 ... 
        - a > b > c > ... 
            - a = heading 
            - b, c, ... = sequential sort-by args
        - SortBy enum to parse names
- [x] `track` command to also accept barcode
- [x] `url list` to accept a query term
- [x] `track` cli command that accepts a url
- [x] better display handling -- super hacky rn 
    - ~~maybe give PackageStatus::Error an item containing the error?~~
    - ~~or properly type the `dict[status, Vec<Package> | Vec<Error>]`~~
- [x] `url add` command to fail if the url is already in the file
- [x] `url remove` to accept substring / regex (so you can remove by barcode)