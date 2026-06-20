# URL management 


## Add a URL
```
❯ packtrack url add example.com/barcode/1234
Added example.com/barcode/1234
```

Add an optional description: 
```
❯ packtrack url add https://jouw.postnl.nl/track-and-trace/POSTNL1-NL-1234AB --description shoes
Added https://jouw.postnl.nl/track-and-trace/POSTNL1-NL-1234AB
```

The description will be displayed in the tracking output:
```
❯ packtrack
╭──────────────────────────────────────────────────────────────────────────────╮
│                              C O M P L E T E D                               │
╰──────────────────────────────────────────────────────────────────────────────╯
[Thu 18 Jun 14:00] PostNL POSTNL1 from Zalando to Packtrack user (shoes)
```

## Remove a URL 
```
❯ packtrack url remove example.com/barcode/1234
Removed urls:
example.com/barcode/1234
```

!!! note 
    Packtrack does a partial match here, so you can pass a fragment of the URL you want to remove (the package barcode is usually good for this).

    If there are multiple partial matches, they will _all_ be removed from the urls file. 
    
    ```
    ❯ packtrack url remove 1234
    Removed urls:
    example.com/barcode/1234
    ```
    
## View the list of tracked URLs
URLs will be displayed in the order they were added (most recent last) and with their description, if they have one:
```sh 
❯ packtrack url list 
https://my.dhlecommerce.nl/home/tracktrace/JVGLOTC0065912345/
https://my.dhlecommerce.nl/home/tracktrace/CF56620412345/1234AB
https://jouw.postnl.nl/track-and-trace/3SFJSY998812345-NL-1234AB
https://jouw.postnl.nl/track-and-trace/3SPTBD402412345-NL-1234AB?language=nl
https://jouw.postnl.nl/track-and-trace/3SYZRF007412345-NL-1234AB
https://jouw.postnl.nl/track-and-trace/POSTNL1-NL-1234AB (shoes)
```

!!! note 
    You can filter the urls: 
    ```
    ❯ packtrack url list dhl
    https://my.dhlecommerce.nl/home/tracktrace/JVGLOTC0065912345/
    https://my.dhlecommerce.nl/home/tracktrace/CF56620412345/1234AB    
    ```