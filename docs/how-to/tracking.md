# Tracking 

## Track all URLs
To track all the URLs in your URLs file and receive a summary, simply run packtrack with no arguments: 
```
❯ packtrack
╭──────────────────────────────────────────────────────────────────────────────╮
│                              C O M P L E T E D                               │
╰──────────────────────────────────────────────────────────────────────────────╯
[Thu 18 Jun 13:30] DHL DHL1 from Bol.com to Packtrack user
  ╰─ Delivered to neighbour at Streetname 420
[Thu 18 Jun 14:00] PostNL POSTNL1 from Zalando to Packtrack user (shoes)
╭──────────────────────────────────────────────────────────────────────────────╮
│                            I N   P R O G R E S S                             │
╰──────────────────────────────────────────────────────────────────────────────╯
PostNL POSTNL2
URL: https://jouw.postnl.nl/track-and-trace/POSTNL2-NL-1234AB
Status: In transit
From: Packtrack user
To: Zalando
ETA: Thu 18 Jun 14:00
ETA window: Thu 18 Jun 12:00 -- 16:00
events:
    [Tue 16 Jun 14:00] Package accepted
    [Wed 17 Jun 14:00] Package sorted at depot
    [Thu 18 Jun 14:00] Package out for delivery
────────────────────────────────────────────────────────────────────────────────
DHL DHL2
URL: https://www.dhl.com/nl-nl/home/tracking.html?submit=1&tracking-id=DHL2
Status: In transit
From: Packtrack user
To: Bol.com
ETA: Thu 18 Jun 14:00
ETA window: Thu 18 Jun 12:00 -- 16:00
events:
    [Tue 16 Jun 14:00] Package accepted
    [Wed 17 Jun 14:00] Package sorted at depot
    [Thu 18 Jun 14:00] Package out for delivery
```

## Track a specific URL
You can also filter for URLs that contain a given string. The package's barcode or tracking code often works here, because it is usually in the URL.
```
❯ packtrack DHL1
╭──────────────────────────────────────────────────────────────────────────────╮
│                              C O M P L E T E D                               │
╰──────────────────────────────────────────────────────────────────────────────╯
[Thu 18 Jun 13:30] DHL DHL1 from Bol.com to Packtrack user
  ╰─ Delivered to neighbour at Streetname 420
```
You can also pass a whole new URL. If packtrack can't find the string in your URLs file, it will assume it is a new URL and track it: 

```
❯ packtrack https://my.dhlecommerce.nl/home/tracktrace/ABCD8
╭──────────────────────────────────────────────────────────────────────────────╮
│                              C O M P L E T E D                               │
╰──────────────────────────────────────────────────────────────────────────────╯
[Sat 19 Jul 13:39] DHL Package ABCD8 from Amazon to Packtrack User
```


By default, delivered packages are shown as a one-liner. In-transit packages are shown in more detail, with events and ETA from the carrier. 

!!! note
    You can use the `-d` / `--delivered` flag to print delivered packages in more detail

## Filter by carrier
Filter for packages carried by PostNL:
```
❯ packtrack --carrier postnl
╭──────────────────────────────────────────────────────────────────────────────╮
│                              C O M P L E T E D                               │
╰──────────────────────────────────────────────────────────────────────────────╯
[Thu 18 Jun 14:00] PostNL POSTNL1 from Zalando to Packtrack user (shoes)
╭──────────────────────────────────────────────────────────────────────────────╮
│                            I N   P R O G R E S S                             │
╰──────────────────────────────────────────────────────────────────────────────╯
PostNL POSTNL2
URL: https://jouw.postnl.nl/track-and-trace/POSTNL2-NL-1234AB
Status: In transit
From: Packtrack user
To: Zalando
ETA: Thu 18 Jun 14:00
ETA window: Thu 18 Jun 12:00 -- 16:00
events:
    [Tue 16 Jun 14:00] Package accepted
    [Wed 17 Jun 14:00] Package sorted at depot
    [Thu 18 Jun 14:00] Package out for delivery
```
!!! note 
    Filters do a partial string match, so `packtrack --carrier post` will also match PostNL. This goes for all the filters.

## Filter by sender
Filter for packages sent by Zalando:
```
❯ packtrack --sender zalando
╭──────────────────────────────────────────────────────────────────────────────╮
│                              C O M P L E T E D                               │
╰──────────────────────────────────────────────────────────────────────────────╯
[Thu 18 Jun 14:00] PostNL POSTNL1 from Zalando to Packtrack user (shoes)
```

## Filter by recipient
Filter for packages sent _to_ Zalando:
```
❯ packtrack --recipient zalando
╭──────────────────────────────────────────────────────────────────────────────╮
│                            I N   P R O G R E S S                             │
╰──────────────────────────────────────────────────────────────────────────────╯
PostNL POSTNL2
URL: https://jouw.postnl.nl/track-and-trace/POSTNL2-NL-1234AB
Status: In transit
From: Packtrack user
To: Zalando
ETA: Thu 18 Jun 14:00
ETA window: Thu 18 Jun 12:00 -- 16:00
events:
    [Tue 16 Jun 14:00] Package accepted
    [Wed 17 Jun 14:00] Package sorted at depot
    [Thu 18 Jun 14:00] Package out for delivery
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