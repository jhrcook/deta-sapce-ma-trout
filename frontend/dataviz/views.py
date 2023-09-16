"""Views."""

from django.http import HttpRequest, HttpResponse
from django.shortcuts import render

from .models import (
    get_latest_stocking_report,
)


# Create your views here.
def index(request: HttpRequest) -> HttpResponse:
    """Root view."""
    trout_report = get_latest_stocking_report()
    dataframe = trout_report.data_as_dataframe()
    context = {
        "trout_report": trout_report,
        "table_colnames": dataframe.columns,
        "table_data": dataframe.iter_rows(),
    }
    return render(
        request=request,
        template_name="dataviz/index.html",
        context=context,
    )
