# MA Trout Stocking Deta App

An application for collecting and analyzing data from the Massachusetts trout stocking program.

## Micros

- [data-scraping](./data-scraping/): scrapes trout stocking data from the website

## To-Do

- front-end for collected data
- notification system for when trout stocking occurs

## Deployment

When trying to deploy locally, the `space push` commands would time-out because building the crates takes too long.
Trying to cross-compile for Linux locally failed.
Therefore, I built a GitHub Action to build the crates on a Linux runner and push to Deta Space from there.
