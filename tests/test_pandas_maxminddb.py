import pandas as pd
import pandas_maxminddb


def test_geolocation():
    ips = pd.DataFrame(
        data={'ip': ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49",
                     "2.30.253.245"]})
    ips.geo.geolocate('ip', './GeoLite2-City.mmdb',
                      ['country', 'city', 'state', 'postcode', 'latitude', 'longitude',
                       'accuracy_radius'])
    assert ips['country'].tolist() == ['US', 'CA', 'BE', 'US', 'GB']
    assert ips['city'].tolist() == ['Houston', 'Montreal', 'Kapellen', 'Northampton', 'London']
    assert ips['state'].tolist() == ['TX', 'QC', 'VLG', 'MA', 'ENG']
    assert ips['latitude'].tolist() == [29.9787, 45.5063, 51.3148, 42.3251, 51.4537]
    assert ips['longitude'].tolist() == [-95.5845, -73.5794, 4.4413, -72.6276, -0.232]
    assert ips['accuracy_radius'].tolist() == [50, 10, 5, 50, 100]


def test_benchmark_maxminddb(benchmark):
    pass


def test_benchmark_pandas_maxminddb(benchmark):
    pass
