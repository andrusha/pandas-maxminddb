# Pandas Maxmind

Provides fast and convenient geolocation bindings for [Pandas](https://pandas.pydata.org/)
Dataframes. Uses [numpy](https://numpy.org/) ndarray's internally to speed it up compared to naively
applying function per column. Based on
the [maxminddb-rust](https://github.com/oschwald/maxminddb-rust).

## Features

- Supports both [MMAP](https://en.wikipedia.org/wiki/Memory_mapping) and in-memory implementations
- Supports parallelism (useful for very big datasets)
- Comes with pre-built wheels, no need to install and maintain external C-library to get (better than) C-performance

## Installation

1. Minimal supported Python is 3.8
2. `pip install pandas_maxminddb`
3. The preferred way is to use precompiled binary wheel, as this requires no toolchain and is
   fastest.
4. If you want to build from source any
   platform [Rust has target](https://doc.rust-lang.org/beta/rustc/platform-support.html) for is
   supported.

### Pre-built wheels

The wheels are built against following `numpy` and `pandas` distributions:

- If you're on Windows / macOS / Linux there is no need to do anything extra.
- If you use ARMv7 (RaspberryPi and such)
  use [PiWheels](https://www.piwheels.org) `--extra-index-url=https://www.piwheels.org/simple`,
  install `libatlas-base-dev` for numpy.
- If you use [musl](https://en.wikipedia.org/wiki/Musl)-based distro like Alpine
  use [Alpine-wheels](https://alpine-wheels.github.io) `--extra-index-url https://alpine-wheels.github.io/index`
  , install `libstdc++` for pandas.

Refer to the [build workflow](./.github/workflows/ci.yml) for details.

| Py   | win x86 | win x64 | macOS x86_64 | macOS AArch64 | linux x86_64 | linux i686 | linux AArch64 | linux ARMv7 | musl linux x86_64 |
|------|---------|---------|--------------|---------------|--------------|------------|---------------|-------------|------------------|
| 3.8  | ✅       | ✅       | ✅            | ✅             | ✅            | ✅          | ✅             | 🚫          | ✅                |
| 3.9  | ✅       | ✅       | ✅            | ✅             | ✅            | ✅          | ✅             | ✅           | 🚫                 |
| 3.10 | 🚫       | ✅       | ✅            | ✅             | ✅            | ✅          | 🚫            | 🚫          | ✅                |

## Usage

By importing `pandas_maxminddb` you add Pandas `geo` extension which allows you to add columns
in-place. This example uses context manager for reader lifetime:

```python
import pandas as pd
from pandas_maxminddb import open_database

ips = pd.DataFrame(data={
    'ip': ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49", "2.30.253.245"]})
with open_database('./GeoLite.mmdb/GeoLite2-City.mmdb') as reader:
    ips.geo.geolocate('ip', reader, ['country', 'city', 'state', 'postcode'])
ips
```

|     | ip              | city        | postcode | state | country |
|-----|-----------------|-------------|----------|-------|---------|
| 0   | 75.63.106.74    | Houston     | 77070    | TX    | US      |
| 1   | 132.206.246.203 | Montreal    | H3A      | QC    | CA      |
| 2   | 94.226.237.31   | Kapellen    | 2950     | VLG   | BE      |
| 3   | 128.119.189.49  | Northampton | 01060    | MA    | US      |
| 4   | 2.30.253.245    | London      | SW15     | ENG   | GB      |

### Without context manager

You can also instantiate reader yourself, eg:

```python
import pandas as pd
from pandas_maxminddb import ReaderMem, ReaderMmap

reader = ReaderMem('./GeoLite.mmdb/GeoLite2-City.mmdb')
ips = pd.DataFrame(data={
    'ip': ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49", "2.30.253.245"]})
ips.geo.geolocate('ip', reader, ['country', 'city', 'state', 'postcode'])
ips
```

### Parallelism

If dataset is big enough, and you have extra cores you might benefit from using them. Currently only `ReaderMem` is supported:

```python
import pandas as pd
from pandas_maxminddb import ReaderMem

reader = ReaderMem('./GeoLite.mmdb/GeoLite2-City.mmdb')
ips = pd.DataFrame(data={
    'ip': ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49", "2.30.253.245"]})
ips.geo.geolocate('ip', reader, ['country', 'city', 'state', 'postcode'], parallel=True)
ips
```

## Benchmarks

- Tested on M1 Max with 1024 chunk size on 100k dataset, refer to [benchmark](./tests/test_pandas_maxminddb.py)

|Name (time in ms)                                                                                                                                                                                                             |Min                |Max                |Mean               |StdDev         |Median             |IQR            |Outliers|OPS          |Rounds|Iterations|
|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------|-------------------|-------------------|---------------|-------------------|---------------|--------|-------------|------|----------|
|test_benchmark_pandas_parallel_mem_maxminddb                                                                                                                                                                                  |52.7588 (1.0)      |57.4206 (1.0)      |54.0573 (1.0)      |1.1782 (1.15)  |53.8497 (1.0)      |1.4194 (1.09)  |4;1     |18.4989 (1.0)|20    |1         |
|test_benchmark_pandas_mmap_maxminddb                                                                                                                                                                                          |240.0050 (4.55)    |244.3257 (4.26)    |242.2177 (4.48)    |1.9017 (1.85)  |243.1021 (4.51)    |3.2122 (2.46)  |2;0     |4.1285 (0.22)|5     |1         |
|test_benchmark_pandas_mem_maxminddb                                                                                                                                                                                           |241.4630 (4.58)    |244.2553 (4.25)    |242.8391 (4.49)    |1.0288 (1.0)   |242.7672 (4.51)    |1.3064 (1.0)   |2;0     |4.1180 (0.22)|5     |1         |
|test_benchmark_c_maxminddb                                                                                                                                                                                                    |1,010.6569 (19.16) |1,055.1080 (18.38) |1,021.3691 (18.89) |18.9273 (18.40)|1,013.3819 (18.82) |12.9544 (9.92) |1;1     |0.9791 (0.05)|5     |1         |
|test_benchmark_python_maxminddb                                                                                                                                                                                               |9,021.2686 (170.99)|9,188.7629 (160.03)|9,071.0055 (167.80)|70.0512 (68.09)|9,039.7811 (167.87)|84.7766 (64.89)|1;0     |0.1102 (0.01)|5     |1         |

## Extending

Due to Dataframe columns being flat arrays and geolocation data coming in a hierarchical format you
might need to provide more mappings to serve your particular use-case. In order to do that follow
Development section to setup your environment and then:

1. Add column name to the [geo_column.rs](./src/geo_column.rs)
2. Add column mapping to the [geolocate.rs](./src/geolocate.rs)

## Development

### Setting up environment

- `git clone --recurse-submodules git@github.com:andrusha/pandas-maxminddb.git`
- `PYTHON_CONFIGURE_OPTS="--enable-shared" asdf install`
- `PYTHON_CONFIGURE_OPTS="--enable-shared" python -m venv .venv`
- `source .venv/bin/activate`
- `pip install nox`
- `nox -s test`
- `PYTHONPATH=.venv/lib/python3.8/site-packages cargo test --no-default-features`

### libmaxminddb

In order to run `nox -s bench` properly you would
need [libmaxminddb](https://github.com/maxmind/libmaxminddb) installed as
per [maxminddb](https://maxminddb.readthedocs.io/en/latest/index.html) instructions prior to
installing Python package, so that C-extension could be benchmarked properly.

On macOS this would require following:

- `brew instal libmaxminddb`
- `PATH="/opt/homebrew/Cellar/libmaxminddb/1.7.1/bin:$PATH" LDFLAGS="-L/opt/homebrew/Cellar/libmaxminddb/1.7.1/lib" CPPFLAGS="-I/opt/homebrew/Cellar/libmaxminddb/1.7.1/include" pip install maxminddb --force-reinstall --verbose --no-cache-dir`
