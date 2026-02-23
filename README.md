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
│                               D E L I V E R E D                              │
╰──────────────────────────────────────────────────────────────────────────────╯
[Tue 25 Mar 13:04] DHL Package ABCD1
[Tue 15 Apr 12:49] DHL Package ABCD2 from Coolblue to Packtrack User
[Fri 02 May 10:53] PostNL Package ABCD3 from Zalando to Packtrack User
[Mon 12 May 10:09] DHL Package ABCD4 from Packtrack User to Coolblue
[Sat 19 Jul 13:39] DHL Package ABCD5 from bol.com to Packtrack User
[Tue 22 Jul 11:58] PostNL Package ABCD6
[Thu 14 Aug 11:45] PostNL Package ABCD7 from Packtrack User to Zalando
```


filter by carrier: 

```
❯ packtrack --carrier dhl
╭──────────────────────────────────────────────────────────────────────────────╮
│                               D E L I V E R E D                              │
╰──────────────────────────────────────────────────────────────────────────────╯
[Tue 25 Mar 13:04] DHL Package ABCD1
[Tue 15 Apr 12:49] DHL Package ABCD2 from Coolblue to Packtrack User
[Mon 12 May 10:09] DHL Package ABCD4 from Packtrack User to Coolblue
[Sat 19 Jul 13:39] DHL Package ABCD5 from bol.com to Packtrack User
```

...or sender: 
```
❯ packtrack --sender coolblue
╭──────────────────────────────────────────────────────────────────────────────╮
│                               D E L I V E R E D                              │
╰──────────────────────────────────────────────────────────────────────────────╯
[Tue 15 Apr 12:49] DHL Package ABCD2 from Coolblue to Packtrack User
```

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