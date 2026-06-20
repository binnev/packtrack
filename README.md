# Packtrack 
A simple CLI for tracking mail packages. See the [documentation](https://binnev.github.io/packtrack/)

## Getting started  

Create your urls file in your home directory: 

```
touch ~/packtrack.urls
```

Add urls you want to track

```
packtrack url add https://my.dhlecommerce.nl/home/tracktrace/ABCD1
```

(Optional) configure your default postcode (this is used to get more info from the carrier)

```
packtrack config set postcode 1234
```

Run packtrack to track all the urls: 
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

See the [tracking how-to page](/how-to/tracking) for more information. 


Consult the help for more options: 

```
❯ packtrack -h
A simple CLI for tracking mail packages

Usage: packtrack [OPTIONS] [URL]
       packtrack <COMMAND>

Commands:
  url     URL management
  config  Configuration
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [URL]  Either a new URL, or a fragment of an existing URL

Options:
  -u, --urls-file <URLS_FILE>          Path to the URLs file
  -s, --sender <SENDER>                Filter by sender
  -c, --carrier <CARRIER>              Filter by postal carrier
  -r, --recipient <RECIPIENT>          Filter by recipient
  -C, --cache-seconds <CACHE_SECONDS>  Max age for cache entries to be reused
  -n, --no-cache                       Don't use the cache (even for delivered packages)
  -d, --delivered                      Display detailed info on delivered packages
  -l, --language <LANGUAGE>            Preferred language (passed to the carrier)
  -p, --postcode <POSTCODE>            Recipient postcode (sometimes required to get full info)
  -v, --verbosity <VERBOSITY>          Set verbosity [default: error]
  -h, --help                           Print help
  -V, --version                        Print version
```