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
- tui spinners when waiting for tasks
- show how long it took to gather each package
- show URL next to errors so we know which error belongs to which URL
- more advanced url file where you can add annotations, for those urls that don't contain your postcode. YAML?
- `url prune` cli command to prune delivered packages 
- cache the responses for each package
    - so you can show the changes over time.
    - global --cache option
    - also reuse recently fetched responses
    - composed Tracker struct with child 
        - Requester 
        - Parser
        - Cacher 
- flag to control package display detail level (brief / detailed)
- `track` command to also accept barcode
- [x] `url list` to accept a query term
- [x] `track` cli command that accepts a url
- [x] better display handling -- super hacky rn 
    - ~~maybe give PackageStatus::Error an item containing the error?~~
    - ~~or properly type the `dict[status, Vec<Package> | Vec<Error>]`~~
- [x] `url add` command to fail if the url is already in the file
- [x] `url remove` to accept substring / regex (so you can remove by barcode)