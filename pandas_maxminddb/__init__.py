from contextlib import contextmanager

import pandas as pd

from .pandas_maxminddb import Reader, __all__, __doc__, mmdb_geolocate  # noqa: F401

__all__ = __all__ + ["GeoAccessor"]


@contextmanager
def open_database(mmdb_path: str) -> Reader:
    yield Reader(mmdb_path)


@pd.api.extensions.register_dataframe_accessor("geo")
class GeoAccessor:
    def __init__(self, pandas_obj: pd.DataFrame):
        self._obj = pandas_obj

    def geolocate(self, ip_column_name: str, reader: Reader, geo_columns: list = None) -> pd.DataFrame:
        """
        :return: appends geolocation information based on the given IP address column
        """
        if geo_columns is None:
            geo_columns = ["country", "city"]

        columns = mmdb_geolocate(self._obj[ip_column_name].values, reader, geo_columns)
        for k, v in columns.items():
            self._obj[k] = v
        return self._obj
