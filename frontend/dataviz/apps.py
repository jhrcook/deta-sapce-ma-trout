"""Apps."""

from django.apps import AppConfig


class DatavizConfig(AppConfig):  # type: ignore
    """Configuration for the 'dataviz' app."""

    default_auto_field = "django.db.models.BigAutoField"
    name = "dataviz"
