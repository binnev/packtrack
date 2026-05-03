# Cache management

## Get the size
Shows the size of the cache in a human-readable format.
``` 
❯ packtrack cache size
3.4 MiB
```

## Prune the cache 
This removes all entries from the cache that are not associated with a URL in the URL store. 

``` 
❯ packtrack cache prune
Removed 94 urls
Cache size reduced from 5.3 MiB to 3.3 MiB
```

## Remove the cache 
Empties the cache (also removes the file, if it is stored on disk).
```
❯ packtrack cache clear
Cleared cache (was 5.3 MiB)
```

## Show the cache location
Shows the location of the cache file on disk.
```
❯ packtrack cache location
/home/username/.cache/packtrack/packtrack-cache.json
```
