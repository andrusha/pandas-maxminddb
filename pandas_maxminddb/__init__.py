import pandas as pd
from .pandas_maxminddb import *
from .pandas_maxminddb import __all__, __doc__

__all__ = __all__ + [
    "GeoAccessor"
]

@pd.api.extensions.register_dataframe_accessor("geo")
class GeoAccessor:
    def __init__(self, pandas_obj: pd.DataFrame):
        self._obj = pandas_obj

    def geolocate(self, ip_column_name: str, mmdb_path: str, geo_columns=["country", "city"]) -> pd.DataFrame:
        """
        :return: appends geolocation information based on the given IP address column
        """
        columns = mmdb_geolocate(self._obj[ip_column_name].values())
        for k, v in columns.items():
            self._obj[k] = v
        return self._obj
