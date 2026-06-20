# Changelog

All notable changes to this project will be documented in this file.

## [3.0.0] - 2026-06-20

### 🚀 Features

- Function to display human-readable bytes


- [**breaking**] Added cache clear method
  - Implemented cache clear for JsonCache. Moved some file logic from CLI into JsonCache.
  - **BREAKING CHANGE:** Cache.save is no longer async. Cache is no longer an async trait. 
- *(UrlStore)* [**breaking**] Added save method

  - **BREAKING CHANGE:** UrlStore implementations no longer persist changes to file in their `add` and `remove` methods. The caller must explicitly call `.save()`. 
- Added DeliveredToNeighbour status and updated package display


- [**breaking**] Made Package.status a field, not a method

  - **BREAKING CHANGE:** Tracker implementations will now have to provide the Package.status field themselves 
- *(DHL)* Implemented DeliveredToNeighbour support


- *(GLS)* Implemented DeliveredToNeighbour support


- *(PostNL)* Implemented DeliveredToNeighbour support


- Added the concept of a "final" package status
  - A package status is considered "final" if there will be no more updates. Previously, Delivered was the only "final" status. Now, however, DeliveredToNeighbour also needs to be treated as final, and there may be other final statuses in future as well -- for example a status to represent a package reaching an international border and being handed over to another carrier.

- *(CLI)* Package display now sorts packages by final/non-final status


- *(CLI)* Oneliner display now includes delivered to neighbour info


- *(Trunkrs)* Added Tracker implementation for Trunkrs


- Added FileHandler trait for easy mocking


- [**breaking**] Replaced SimpleUrlStore and JsonUrlStore with FileUrlStore


- [**breaking**] Replaced JsonCache with FileCache


- [**breaking**] Replaced cli settings module with FileSettingsManager


### 🐛 Bug Fixes

- JsonCache size works when there's no file


- *(CLI)* Refer to "completed" packages instead of "delivered" packages


- *(GLS)* Fixed false positive delivered to neighbour


- UrlStore implementations now use FileHandler


- *(CLI)* Config update now displays updated values


### 🚜 Refactor

- Cached tracker


- Rename cli -> main


- *(CLI)* Separate file for cache commands


- *(CLI)* Separate file for URL commands


- *(CLI)* Separate file for config commands


- *(CLI)* Separate file for track command


- Grouped tracker implementations


- Grouped UrlStore implementations


- Rename tracker traits file


- *(UrlStore)* Separated errors and models
  - Also added UrlError as a crate::Error member

- *(CLI)* Move commands to folder


- [**breaking**] Removed cli urls module and instead used FileUrlStore directly


### 📚 Documentation

- Update tracking how-to


- URL management and readme updates


### 🧪 Testing

- Added test which generates some docs


### ⚙️ Miscellaneous Tasks

- Removed dead code


## [2.8.0] - 2026-05-03

### 🚀 Features

- *(CLI)* Cache commands to show location and clear cache


### 📚 Documentation

- Cache clear and location commands


## [2.7.0] - 2026-05-02

### 🚀 Features

- Cache size command


- Cache prune command


### 🚜 Refactor

- Split up cache into module


### 📚 Documentation

- Documentation for cache commands


### 🧪 Testing

- Added dhl mock for delivered to neighbours


- Added fedex mocks


- Added dpd mocks


- Fix flaky test


### ⚙️ Miscellaneous Tasks

- Update flake


## [2.6.1] - 2026-04-03

### 🐛 Bug Fixes

- PostNL now correctly handles letterbox deliveries


## [2.6.0] - 2026-03-15

### 🚀 Features

- GLS events now mention the neighbour the package was delivered to


## [2.5.0] - 2026-03-10

### 🚀 Features

- *(GLS)* Handle gls-group.eu URLs


### 🐛 Bug Fixes

- *(postnl)* If company and person name are missing, show address instead


## [2.4.0] - 2026-02-23

### 🚀 Features

- Added UrlStore trait and JSON implementation


- *(cli)* Verbosity arg now accepts info|debug|etc and 0|1|2|etc


- *(cli)* Added file parameter to url commands


- Added SimpleUrlStore for plain text files
  - Now we just need to have the urls.rs detect the UrlStore type depending on the file extension

- Url filtering also searches description


- Url -f flag, main -u flag, and SimpleUrlStore
  - Added the --file flag to the url management commands. Added the --description flag to the url add command. Got SimpleUrlStore working in a backwards-compatible way. Added the --urls-file flag to the main command. The settings module now returns a UrlStore object.

- *(cli)* URL description is displayed in CLI output


- Better error display


- *(cli)* Nicer headings and lines


- *(cli)* Better heading text


### 🐛 Bug Fixes

- *(postnl)* Fixed a deserialization bug when a datetime was missing


- *(cli)* Tracker args are no longer available for subcommands


- Rename urls -f flag to -u to match main command


- *(cli)* Headings are only displayed if they contain packages


### 🚜 Refactor

- Rename cli/utils -> cli/display


- *(cli)* Simplified job display


### 📚 Documentation

- Update display


### 🧪 Testing

- SimpleUrlStore tests


- Fix settings test


## [2.3.0] - 2025-11-16

### 🚀 Features

- *(tracker)* Raise error on bad status code


- Retry without default postcode
  - If we get an error from the carrier API, we now try again without the default postcode.

### 🐛 Bug Fixes

- Only retry without default postcode if it's a client error


### 🚜 Refactor

- Cached tracker logic


## [2.2.1] - 2025-11-08

### 🐛 Bug Fixes

- *(settings)* Fixed broken string parsing


- *(settings)* We now check if urls_file is a valid path


- *(cli)* Removed redundant settings load


- *(urls)* Made urls module stateless
  - Removed references to settings in the urls module

- *(cli)* Config command doesn't load settings twice anymore


## [2.2.0] - 2025-11-07

### 🚀 Features

- Added preferred_language option to tracker trait


- Added language and postcode CLI options


- *(postnl)* Added language support


- Added no_cache CLI option


- Added CLI option to show delivered packages in detail


### 🐛 Bug Fixes

- Removed unused display_format option
  - This should be a cli-specific setting anyway

- Removed unused use_cache setting


### 📚 Documentation

- Added docs for new CLI options


## [2.1.0] - 2025-10-02

### 🚀 Features

- *(cli)* Shorter arg aliases


## [2.0.0] - 2025-09-16

### 🚀 Features

- [**breaking**] Move settings and url management from core to cli


### 🐛 Bug Fixes

- *(postnl)* More robust ETA window handling for PostNL
  - PostNL tracker now takes ETA window from multiple sources

- *(cli)* Multiline formatting for delivered packages


- *(postnl)* More robust barcode + postcode extraction from urls


- *(cli)* Display recipient for in-transit packages


- *(cache)* Bad cache entries no longer prevent us fetching a fresh value


### 📚 Documentation

- Guides for tracking and url management


### 🧪 Testing

- *(postnl)* More tests


## [1.0.2] - 2025-07-21

### 🐛 Bug Fixes

- *(api)* Fixed tracking a url that isn't in the urls file


## [1.0.1] - 2025-07-11

### 🐛 Bug Fixes

- Remove unstable let_chains feature


### ⚙️ Miscellaneous Tasks

- Gitignore


## [1.0.0] - 2025-07-11

### 🚀 Features

- Implemented filtering


- Version option in cli


- [**breaking**] Refactor to lib + bin


- Errors are now printed with the corresponding URL


### 📚 Documentation

- Fix reference docs link


### ⚙️ Miscellaneous Tasks

- Remove TODO.md


## [0.2.2] - 2025-06-28

### 🐛 Bug Fixes

- Docs repo link


## [0.2.1] - 2025-06-28

### 🐛 Bug Fixes

- Readme docs link and docs fixes


## [0.2.0] - 2025-06-28

### 🚀 Features

- Installed rust template project


### 📚 Documentation

- Update docs; moved dev docs to invisible folder.


- Switch readme position


- Update readme


### 🎨 Styling

- Cargo fmt


## [0.1.0] - 2024-11-21

<!-- generated by git-cliff -->
