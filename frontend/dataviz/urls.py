"""URLs."""

from django.urls import path

from . import views

urlpatterns = [
    path("", views.current_table, name="current_table"),
    path("current-table", views.current_table, name="current_table"),
    path("current-map", views.current_map, name="current_map"),
    path("analysis", views.analysis, name="analysis"),
]
