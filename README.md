# MA Trout Stocking Deta App

**An application for collecting and analyzing data from the Massachusetts trout stocking program.**

## Micros

### "data-scraping"

Periodically collects, validates, and stores trout stocking data.
The crate ['trout_scraping'](./trout_scraping/) contains the data model and scraping system, while the crate ['trout_scraping_server'](./trout_scraping_server/) contains the web server for executing the data-scraping and pushing the results to a Deta Base.

### 'front-end'

A Django website that presents the data.

## To-Do

- front-end for collected data
- notification system for when trout stocking occurs

## Tutorials

- [ ] Building and pushing to Deta Space from GH Action
- [ ] Cargo Workspace for Deta Space application

## Deployment

When trying to deploy locally, the `space push` commands would time-out because building the crates takes too long.
Trying to cross-compile for Linux locally failed.
