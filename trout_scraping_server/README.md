# Web-server to collect and store trout stocking data from the MA DCR website

## Overview

This is intended to be run as a Micro on Deta Space and called periodically to collect and store data for other analyses and notification systems.
There are three endpoints:

1. root (`/`): Just prints a quick message "Trout web-scraping Micro"
1. demo (`/demo`): Scrapes and returns the current trout stocking data
1. Deta Space actions (`/__space/v0/actions`): Called by Deta Space to scrape and store the current trout stocking data
