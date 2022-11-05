from contextlib import contextmanager

import pandas as pd

from .pandas_maxminddb import Reader, ReaderMem, ReaderMmap, mmdb_geolocate

__all__ = ["open_database", "GeoAccessor", "Reader", "ReaderMem", "ReaderMmap"]


@contextmanager
def open_database(mmdb_path: str, mmap=False) -> Reader:
    """
    If you want to manage lifetime of the object yourself,
     then instantiate ReaderMem / ReaderMmap yourself

    :param mmdb_path: path maxmind db
    :param mmap: use memory mapping or not, useful for big files and few lookups
    :return: corresponding context-managed Reader, which can be used with `with` statement
    """

    if mmap:
        yield ReaderMmap(mmdb_path)
    else:
        yield ReaderMem(mmdb_path)


@pd.api.extensions.register_dataframe_accessor("geo")
class GeoAccessor:
    """
    Defines Dataframe extension, which can be accessible as `some_df.geo.geolocate`
    """

    def __init__(self, pandas_obj: pd.DataFrame):
        self._obj = pandas_obj

    def geolocate(
        self, ip_column_name: str, reader: Reader, geo_columns: list = None, parallel=False, parallel_chunk_size=1024
    ) -> pd.DataFrame:
        """
        :param ip_column_name: name of the dataframe column containing IPs, malformed IPs are ignored
        :param reader: one of the reader classes
        :param geo_columns: list of columns to lookup
        :param parallel: if lookups should be done in parallel (uses all the available cores)
        :param parallel_chunk_size: size of the job into which ip list is split for parallel processing
        :return: appends geolocation information based on the given IP address column
        """
        if geo_columns is None:
            geo_columns = ["country", "city"]

        columns = mmdb_geolocate(self._obj[ip_column_name].values, reader, geo_columns, parallel, parallel_chunk_size)
        for k, v in columns.items():
            self._obj[k] = v
        return self._obj
