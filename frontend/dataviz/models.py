"""Models."""

from typing import Any

from deta import Deta

# Deta client.
deta = Deta()


def get_trout_data() -> dict[str, Any]:
    """Get a single trout data."""
    db = deta.Base("trout-stocking")
    return db.get("5tg8t7zs9xcf")  # type: ignore
