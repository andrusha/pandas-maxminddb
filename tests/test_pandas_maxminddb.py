import pandas as pd
import pandas_maxminddb
import pytest

GEOLITE_CITY_MMDB = "./GeoLite.mmdb/GeoLite2-City.mmdb"


def test_geolocation():
    ips = pd.DataFrame(
        data={"ip": ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49", "2.30.253.245"]}
    )
    with pandas_maxminddb.open_database(GEOLITE_CITY_MMDB) as reader:
        ips.geo.geolocate(
            "ip", reader, ["country", "city", "state", "postcode", "latitude", "longitude", "accuracy_radius"]
        )

    assert ips["country"].tolist() == ["US", "CA", "BE", "US", "GB"]
    assert ips["city"].tolist() == ["Houston", "Montreal", "Kapellen", "Northampton", "London"]
    assert ips["state"].tolist() == ["TX", "QC", "VLG", "MA", "ENG"]
    assert ips["latitude"].tolist() == [29.9787, 45.5063, 51.3148, 42.3251, 51.4537]
    assert ips["longitude"].tolist() == [-95.5845, -73.5794, 4.4413, -72.6276, -0.232]
    assert ips["accuracy_radius"].tolist() == [20, 10, 10, 50, 100]


@pytest.fixture
def random_ips():
    import random

    return [
        f"{random.randint(0, 255)}.{random.randint(0, 255)}.{random.randint(0, 255)}.{random.randint(0, 255)}"
        for _ in range(0, 100_000)
    ]


def py_get_geo(reader, ip):
    def none_if_exception(m):
        try:
            return m()
        except (KeyError, TypeError):
            return None

    try:
        geo = reader.get(ip)

        return dict(
            country=none_if_exception(lambda: geo["country"]["names"]["en"]),
            state=none_if_exception(lambda: geo["subdivisions"][0]["names"]["en"]),
            city=none_if_exception(lambda: geo["city"]["names"]["en"]),
            postcode=none_if_exception(lambda: geo["postal"]["code"]),
            longitude=none_if_exception(lambda: float(geo["location"]["longitude"])),
            latitude=none_if_exception(lambda: float(geo["location"]["latitude"])),
            accuracy_radius=none_if_exception(lambda: int(geo["location"]["accuracy_radius"])),
        )
    except (KeyError, ValueError):
        return None


def py_geolocate(reader, df):
    geos = df["ip"].map(lambda ip: py_get_geo(reader, ip))
    geos = pd.json_normalize(geos)
    return pd.concat([df, geos], axis=1)


def test_benchmark_python_maxminddb(benchmark, random_ips):
    import maxminddb

    ips = pd.DataFrame(data={"ip": random_ips})
    with maxminddb.open_database(GEOLITE_CITY_MMDB, maxminddb.MODE_MMAP) as reader:
        benchmark(py_geolocate, reader, ips)


def test_benchmark_c_maxminddb(benchmark, random_ips):
    import maxminddb

    ips = pd.DataFrame(data={"ip": random_ips})
    with maxminddb.open_database(GEOLITE_CITY_MMDB, maxminddb.MODE_MMAP_EXT) as reader:
        benchmark(py_geolocate, reader, ips)


def test_benchmark_pandas_mem_maxminddb(benchmark, random_ips):
    ips = pd.DataFrame(data={"ip": random_ips})
    with pandas_maxminddb.open_database(GEOLITE_CITY_MMDB) as reader:
        benchmark(
            ips.geo.geolocate,
            "ip",
            reader,
            ["country", "city", "state", "postcode", "latitude", "longitude", "accuracy_radius"],
        )


def test_benchmark_pandas_parallel_mem_maxminddb(benchmark, random_ips):
    ips = pd.DataFrame(data={"ip": random_ips})
    with pandas_maxminddb.open_database(GEOLITE_CITY_MMDB) as reader:
        benchmark(
            ips.geo.geolocate,
            "ip",
            reader,
            ["country", "city", "state", "postcode", "latitude", "longitude", "accuracy_radius"],
            True,
            1024,
        )


def test_benchmark_pandas_mmap_maxminddb(benchmark, random_ips):
    ips = pd.DataFrame(data={"ip": random_ips})
    with pandas_maxminddb.open_database(GEOLITE_CITY_MMDB, mmap=True) as reader:
        benchmark(
            ips.geo.geolocate,
            "ip",
            reader,
            ["country", "city", "state", "postcode", "latitude", "longitude", "accuracy_radius"],
        )
