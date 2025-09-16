# URL management 


## Add a URL
```
❯ packtrack url add example.com/barcode/1234
Added example.com/barcode/1234
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
```sh 
❯ packtrack url list 
https://my.dhlecommerce.nl/home/tracktrace/JVGLOTC0065912345/
https://my.dhlecommerce.nl/home/tracktrace/CF56620412345/1234AB
https://jouw.postnl.nl/track-and-trace/3SFJSY998812345-NL-1234AB
https://jouw.postnl.nl/track-and-trace/3SPTBD402412345-NL-1234AB?language=nl
https://jouw.postnl.nl/track-and-trace/3SYZRF007412345-NL-1234AB
https://jouw.postnl.nl/track-and-trace/3SDMDN031112345-NL-1234AB
```

!!! note 
    You can filter the urls: 
    ```
    ❯ packtrack url list dhl
    https://my.dhlecommerce.nl/home/tracktrace/JVGLOTC0065912345/
    https://my.dhlecommerce.nl/home/tracktrace/CF56620412345/1234AB    
    ```