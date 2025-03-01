"""Models."""

from collections.abc import Mapping
from datetime import datetime
from enum import Enum
from functools import cache
from typing import Annotated, Optional, Union

import polars as pl
from dateutil import parser
from deta import Deta
from loguru import logger
from pydantic import BaseModel
from pydantic.functional_validators import BeforeValidator

# Deta client.
deta = Deta()


def _parse_datetime(v: str) -> datetime:
    return parser.parse(v)


class Timestamp(BaseModel):
    """Data timestamp."""

    datetime: Annotated[datetime, BeforeValidator(_parse_datetime)]
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
    data: dict[str, list[Optional[str]]]
    timestamp: Timestamp

    def data_as_dataframe(self) -> pl.DataFrame:
        """Data in a table layout."""
        return pl.DataFrame(self.data)

    def __hash__(self) -> int:
        """Hash for an object based on the timestamp."""
        return hash(self.timestamp.datetime)


class DetaBase(str, Enum):
    """Deta Base ID."""

    TROUT_STOCKING = "trout-stocking"
    TROUT_STOCKING_RAW = "trout-stocking-raw"


DetaBaseQuery = Mapping[str, Union[str, int, float]]


def retrieve_stocking_data(
    query: Optional[Union[DetaBaseQuery, list[DetaBaseQuery]]] = None
) -> list[TroutStockingReport]:
    """Retrieve all trout stocking data.

    Args:
        query (Optional[Union[DetaBaseQuery, list[DetaBaseQuery]]], optional): Optional queries to filter the Deta Base. Defaults to None.

    Returns:
        list[TroutStockingReport]: List of all trout stocking reports.
    """  # noqa: E501
    db = deta.Base(DetaBase.TROUT_STOCKING.value)
    res = db.fetch(query)
    all_items = res.items
    # Continue fetching until "res.last" is None.
    while res.last:
        res = db.fetch(query, last=res.last)
        all_items += res.items
    return [TroutStockingReport(**d) for d in all_items]


@cache
def get_latest_stocking_report(reload: bool = False) -> TroutStockingReport:
    """Retrieve the most recent trout stocking report."""
    if reload:
        logger.info("Reloading data (does nothing at the moment...)")
    trout_reports = retrieve_stocking_data()
    trout_reports.sort(key=lambda tr: tr.timestamp.datetime)
    return trout_reports[-1]
