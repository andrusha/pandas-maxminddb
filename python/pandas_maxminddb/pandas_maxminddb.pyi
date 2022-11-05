from typing import Dict, List

import numpy

class Reader:
    """
    Abstract superclass of all the readers, can not be instantiated,
    should be used as a type hint
    """

class ReaderMem(Reader):
    """
    Loads MMDB in-memory, required when parallel processing is used
    """

    def __init__(self, mmdb_path: str) -> None:
        """
        :param mmdb_path: path to maxmind db file
        """

class ReaderMmap(Reader):
    """
    Uses memory map to read the db, so only the records you're accessing are read from disk.
    Useful when memory is limited and few lookups are made
    """

    def __init__(self, mmdb_path: str) -> None:
        """
        :param mmdb_path: path to maxmind db file
        """

def mmdb_geolocate(
    ips: numpy.ndarray, reader: Reader, columns: List[str], parallel: bool, parallel_chunk_size: int
) -> Dict[str, numpy.ndarray]:
    """

    :param ips: ndarray of ip strings
    :param reader: one of the reader subclasses
    :param columns: list of columns to fetch
    :param parallel: if processing should be done in parallel
    :param parallel_chunk_size: chunk size for ips to be split for parallel processing
    :return: dict with keys being columns and values ndarray of lookup results
    """
