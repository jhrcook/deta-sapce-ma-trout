"""Views."""

from django.http import HttpRequest, HttpResponse
from django.shortcuts import render

from .models import (
    get_latest_stocking_report,
)


# Create your views here.
def current_table(request: HttpRequest) -> HttpResponse:
    """Current stocking report table view."""
    trout_report = get_latest_stocking_report()
    dataframe = trout_report.data_as_dataframe()
    context = {
        "trout_report": trout_report,
        "table_colnames": dataframe.columns,
        "table_data": dataframe.iter_rows(),
    }
    return render(
        request=request,
        template_name="dataviz/stocking_report_table.html",
        context=context,
    )


def current_map(request: HttpRequest) -> HttpResponse:
    """Map of current stocking report view."""
    return render(
        request=request,
        template_name="dataviz/stocking_report_map.html",
        context={},
    )


def analysis(request: HttpRequest) -> HttpResponse:
    """Analysis of the stocking data view."""
    return render(
        request=request,
        template_name="dataviz/analysis.html",
        context={},
    )
