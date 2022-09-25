import pandas as pd
from pandas_maxminddb import GeoAccessor

def test_geolocation():
    ips = pd.DataFrame(
        data={'ip': ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49", "2.30.253.245"]})
    ips.geo.geolocate('ip', './GeoLite2-City.mmdb')
    assert ips['city'].tolist() == ['Something']


def test_benchmark_maxminddb(benchmark):
    pass


def test_benchmark_pandas_maxminddb(benchmark):
    pass
