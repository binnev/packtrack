# Tracking 

## Track all URLs 
To track all the URLs in your URLs file and receive a summary, simply run packtrack with no arguments: 

```
❯ packtrack 
================================================================================
                            D E L I V E R E D                                
================================================================================
[Tue 25 Mar 13:04] DHL Package ABCD1
[Tue 15 Apr 12:49] DHL Package ABCD2 from Coolblue to Packtrack User
[Fri 02 May 10:53] PostNL Package ABCD3 from Zalando to Packtrack User
[Mon 12 May 10:09] DHL Package ABCD4 from Packtrack User to Coolblue
[Sat 19 Jul 13:39] DHL Package ABCD5 from bol.com to Packtrack User
[Tue 22 Jul 11:58] PostNL Package ABCD6
[Thu 14 Aug 11:45] PostNL Package ABCD7 from Packtrack User to Zalando
================================================================================
                            I N T R A N S I T                                
================================================================================

================================================================================
                                E R R O R S                                   
================================================================================
```

## Track a specific URL 
You can also filter for URLs that contain a given string. The package's barcode or tracking code often works here, because it is usually in the URL.

```
❯ packtrack ABCD5
================================================================================
                            D E L I V E R E D                                
================================================================================
[Sat 19 Jul 13:39] DHL Package ABCD5 from bol.com to Packtrack User
================================================================================
                            I N T R A N S I T                                
================================================================================

================================================================================
                                E R R O R S                                   
================================================================================
```

You can also pass a whole new URL. If packtrack can't find the string in your URLs file, it will assume it is a new URL and track it: 

```
❯ packtrack https://my.dhlecommerce.nl/home/tracktrace/ABCD8
================================================================================
                            D E L I V E R E D                                
================================================================================
[Sat 19 Jul 13:39] DHL Package ABCD8 from Amazon to Packtrack User
================================================================================
                            I N T R A N S I T                                
================================================================================

================================================================================
                                E R R O R S                                   
================================================================================
```


By default, delivered packages are shown as a one-liner. In-transit packages are shown in more detail, with events and ETA from the carrier. 

!!! note
    You can use the `-d` / `--delivered` flag to print delivered packages in more detail

## Filter by carrier 
Filter for packages carried by PostNL: 
```
❯ packtrack --carrier postnl
================================================================================
                            D E L I V E R E D                                
================================================================================
[Tue 22 Jul 11:58] PostNL Package ABCD6
================================================================================
                            I N T R A N S I T                                
================================================================================
PostNL Package ABCD3 from Zalando to Packtrack User
events:
    [Today 12:16] Pre-alerted shipment enriched by PostNL management.
    [Today 12:16] Shipment expected, but not yet arrived or processed at PostNL
    [Today 12:16] Delivery can be changed
    [Today 19:35] Shipment received by PostNL
    [Today 19:37] Delivery can be changed

--------------------------------------------------------------------------------
PostNL Package ABCD7 from Packtrack User to Zalando
events:
    [Today 11:12] Pre-alerted shipment enriched by PostNL management.
    [Today 11:12] Shipment expected, but not yet arrived or processed at PostNL
    [Today 11:12] Delivery can be changed

================================================================================
                                E R R O R S                                   
================================================================================
```

!!! note 
    Filters do a partial string match, so `packtrack --carrier post` will also match PostNL. This goes for all the filters.

## Filter by sender 
Filter for packages sent by Coolblue:
```
❯ packtrack --sender coolblue
================================================================================
                            D E L I V E R E D                                
================================================================================
[Tue 15 Apr 12:49] DHL Package ABCD2 from Coolblue to Packtrack User
================================================================================
                            I N T R A N S I T                                
================================================================================

================================================================================
                                E R R O R S                                   
================================================================================
```

## Filter by recipient 
Filter for packages sent _to_ Coolblue: 
```
❯ packtrack --recipient coolblue 
================================================================================
                            D E L I V E R E D                                
================================================================================
[Mon 12 May 10:09] DHL Package ABCD4 from Packtrack User to Coolblue
================================================================================
                            I N T R A N S I T                                
================================================================================

================================================================================
                                E R R O R S                                   
================================================================================

```


## Caching 
To speed things up, packtrack reuses cached responses where possible. Undelivered packages are loaded from the cache if the cache entry is less than 30s old. This time window is called the "cache lifetime". To override this value, use the `-c` flag to pass a new cache lifetime in seconds. 

This will reuse cache entries from the past 10 minutes:
```
packtrack -c 600
```

This will force packtrack to always fetch a fresh value: 
```
packtrack -c 0 
```

Delivered packages are _always_ loaded from the cache, because they are unlikely to change. 

!!! note 
    To disable the cache (even for delivered packages), use the `-n`/`--no-cache` option.

## Language 
The `-l`/`--language` option can be used to specify a preferred language. Pass an [ISO 639](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes) language code e.g. "en": 

```
packtrack -l en 
```

Packtrack will pass this to the carrier API, if it supports it.