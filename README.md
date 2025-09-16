# Packtrack 
A simple CLI for tracking mail packages. See the [documentation](https://binnev.github.io/packtrack/ABCD/)

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


filter by carrier: 

```
❯ packtrack --carrier dhl
================================================================================
                            D E L I V E R E D                                
================================================================================
[Tue 25 Mar 13:04] DHL Package ABCD1
[Tue 15 Apr 12:49] DHL Package ABCD2 from Coolblue to Packtrack User
[Mon 12 May 10:09] DHL Package ABCD4 from Packtrack User to Coolblue
[Sat 19 Jul 13:39] DHL Package ABCD5 from bol.com to Packtrack User
================================================================================
                            I N T R A N S I T                                
================================================================================

================================================================================
                                E R R O R S                                   
================================================================================
```

...or sender: 
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

Consult the help for more options: 

```
❯ packtrack -h
A simple CLI for tracking mail packages

Usage: packtrack [OPTIONS] [URL] [COMMAND]

Commands:
url     URL management
config  Configuration
help    Print this message or the help of the given subcommand(s)

Arguments:
[URL]  Either a new URL, or a fragment of an existing URL

Options:
-v, --verbosity...                   Set verbosity. `-v` = 1, `-vvv` = 3
-c, --cache-seconds <CACHE_SECONDS>  Max age for cache entries to be reused
    --sender <SENDER>                Filter by sender
    --carrier <CARRIER>              Filter by postal carrier
    --recipient <RECIPIENT>          Filter by recipient
-h, --help                           Print help
-V, --version                        Print version
```