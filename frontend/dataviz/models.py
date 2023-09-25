"""Models."""
from __future__ import annotations

import os
from collections.abc import Mapping
from datetime import datetime
from enum import Enum
from functools import cache
from typing import Annotated, Any, Union

import polars as pl
import requests
from dateutil import parser
from deta import Deta
from loguru import logger
from pydantic import BaseModel, NonNegativeInt
from pydantic.functional_validators import BeforeValidator

# Deta client.
deta = Deta()

DetaBaseQuery = Mapping[str, Union[str, int, float]]


def deta_base_fetch_all(db: DetaBase, query: DetaBaseQuery | None = None) -> list[Any]:
    """Fetch all data from a Deta Base."""
    base = deta.Base(db.value)
    res = base.fetch(query)
    all_items: list[Any] = res.items
    # Continue fetching until "res.last" is None.
    while res.last:
        res = base.fetch(query, last=res.last)
        all_items += res.items
    return all_items


class DetaBase(str, Enum):
    """Deta Base."""

    TROUT_STOCKING_RAW = "trout-stocking-raw"
    TROUT_STOCKING = "trout-stocking"
    TROUT_STOCKING_INDEX = "trout-stocking-index"


def _parse_datetime(v: str) -> datetime:
    return parser.parse(v)


class Timestamp(BaseModel):
    """Data timestamp."""

    datetime: Annotated[datetime, BeforeValidator(_parse_datetime)]
    timestamp: int
    year: int
    month: int
    day: int
    hour: int
    min: int  # noqa: A003
    sec: int


class TroutStockingReport(BaseModel):
    """Trout stocking report."""

    key: str
    version: str
    req_id: str
    status: str
    sig: str
    data: dict[str, list[str | None]]
    timestamp: Timestamp

    def data_as_dataframe(self) -> pl.DataFrame:
        """Data in a table layout."""
        return pl.DataFrame(self.data)

    def __hash__(self) -> int:
        """Hash for an object based on the timestamp."""
        return hash(self.key)

    @staticmethod
    def get_report(key: str) -> TroutStockingReport:
        """Get a stocking report.

        Args:
            key (str | UUID): Stocking report key.

        Returns:
            TroutStockingReport: Trout stocking report.
        """
        return TroutStockingReport(**deta.Base(DetaBase.TROUT_STOCKING.value).get(key))


class TroutStockingReportMetadata(BaseModel):
    """Metadata for a single trout stocking report."""

    key: str
    timestamp: NonNegativeInt
    num_stocked_locations: NonNegativeInt


class TroutStockingReportIndex(BaseModel):
    """Index for the trout stocking report database."""

    data: list[TroutStockingReportMetadata]

    @staticmethod
    def retrieve() -> TroutStockingReportIndex:
        """Retrieve the TroutStockingReportIndex."""
        return TroutStockingReportIndex(
            **{"data": deta_base_fetch_all(DetaBase.TROUT_STOCKING_INDEX)}
        )


@cache
def get_most_recently_collected_stocking_report() -> TroutStockingReport:
    """Get the most recently collectin stocking report."""
    logger.debug("Getting most recent stocking report.")
    index = TroutStockingReportIndex.retrieve()
    max_ts = max([i.timestamp for i in index.data])
    newest_report = next(iter(filter(lambda i: i.timestamp == max_ts, index.data)))
    logger.debug(f"Newest report: {newest_report}")
    return TroutStockingReport.get_report(newest_report.key)


@cache
def get_current_stocking_report(reload: bool = False) -> TroutStockingReport:
    """Retrieve the most recent trout stocking report."""
    logger.debug("Retrieving current stocking report.")
    if reload:
        logger.warning("Reloading data (does nothing at the moment...)")

    if (hostname := os.getenv("DETA_SPACE_APP_HOSTNAME")) is None:
        logger.error("Could not get Deta App hostname.")
        return get_most_recently_collected_stocking_report()

    if (api_key := os.getenv("DETA_API_KEY")) is None:
        logger.error("Cannot get Deta API key.")
        return get_most_recently_collected_stocking_report()

    url = f"https://{hostname}/data-scraping/demo"
    headers: Mapping[str, str] = {"x-api-key": api_key}
    res = requests.get(url, headers=headers, timeout=(3, 5))

    if res.status_code == requests.codes.ok:
        try:
            return TroutStockingReport(**res.json())
        except BaseException as err:
            logger.error("Error during data validation.")
            logger.error(f"Exception: {err}")
    logger.error(f"Failed data request: {res.status_code}.")
    logger.debug(f"Failed data request: {res.json()}.")
    return get_most_recently_collected_stocking_report()
