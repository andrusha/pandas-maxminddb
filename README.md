# Pandas Maxmind

Provides fast and convenient geolocation bindings for [Pandas](https://pandas.pydata.org/)
Dataframes. Uses [numpy](https://numpy.org/) ndarray's internally to speed it up compared to naively
applying function per column. Based on
the [maxminddb-rust](https://github.com/oschwald/maxminddb-rust).

## Installation

1. Minimal supported Python is 3.8
2. `pip install pandas_maxminddb`
3. The preferred way is to use precompiled binary wheel, as this requires no toolchain and is
   fastest.
4. If you want to build from source any
   platform [Rust has target](https://doc.rust-lang.org/beta/rustc/platform-support.html) for is
   supported.

### Pre-build wheels

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
in-place

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

## Benchmarks

| Name (time in ms)               | Min                | Max                | Mean               | StdDev           | Median             | IQR              | Outliers | OPS           | Rounds | Iterations |
|---------------------------------|--------------------|--------------------|--------------------|------------------|--------------------|------------------|----------|---------------|--------|------------|
| test_benchmark_pandas_maxminddb | 273.2588 (1.0)     | 284.8850 (1.0)     | 280.4760 (1.0)     | 4.5448 (1.0)     | 281.6831 (1.0)     | 5.9721 (1.0)     | 1;0      | 3.5654 (1.0)  | 5      | 1          |
| test_benchmark_c_maxminddb      | 986.0314 (3.61)    | 1,002.4413 (3.52)  | 995.7461 (3.55)    | 8.3891 (1.85)    | 1,001.3420 (3.55)  | 15.1085 (2.53)   | 2;0      | 1.0043 (0.28) | 5      | 1          |
| test_benchmark_python_maxminddb | 9,011.4650 (32.98) | 9,286.9635 (32.60) | 9,081.2087 (32.38) | 117.9029 (25.94) | 9,020.5363 (32.02) | 114.9376 (19.25) | 1;0      | 0.1101 (0.03) | 5      | 1          |

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
