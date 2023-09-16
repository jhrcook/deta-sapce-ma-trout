"""Views."""

from django.http import HttpRequest, HttpResponse
from django.shortcuts import render

from .models import get_trout_data


# Create your views here.
def index(request: HttpRequest) -> HttpResponse:
    """Root view."""
    data = get_trout_data()
    context = {
        "title": "Trout Data Visualization",
        "trout_data": data,
    }
    return render(
        request=request,
        template_name="dataviz/index.html",
        context=context,
    )
