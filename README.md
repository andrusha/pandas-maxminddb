# Pandas Maxmind

Provides fast and convenient geolocation bindings for Pandas Dataframes. Uses numpy ndarray's internally to speed it up compared to naively applying function per column.  

## Usage

By importing `pandas_maxminddb` you add Pandas `geo` extension which allows you to add columns in-place

```python
import pandas as pd
import pandas_maxminddb

ips = pd.DataFrame(data={'ip': ["75.63.106.74", "132.206.246.203", "94.226.237.31", "128.119.189.49", "2.30.253.245"]})
ips.geo.geolocate('ip', './GeoLite2-City.mmdb', ['country', 'city', 'state', 'postcode'])
ips
```

|     |ip             |city       |postcode|state|country|
|-----|---------------|-----------|--------|-----|-------|
| 0   |75.63.106.74   |Houston    |77070   |TX   |US     |
| 1   |132.206.246.203|Montreal   |H3A     |QC   |CA     |
| 2   |94.226.237.31  |Kapellen   |2950    |VLG  |BE     |
| 3   |128.119.189.49 |Northampton|01060   |MA   |US     |
| 4   |2.30.253.245   |London     |SW15    |ENG  |GB     |


## Todo
- [ ] Add type annotations https://maturin.rs/project_layout.html
- [ ] Distribute for multiple python versions https://pyo3.rs/v0.17.1/building_and_distribution.html
- [ ] Figure out GIL https://pyo3.rs/v0.17.1/types.html
- [ ] Add error handling

## Develop

- `git clone --recurse-submodules git@github.com:andrusha/pandas-maxminddb.git`
- `asdf install`
- `python -m venv .venv`
- `source .venv/bin/activate`
- `pip install nox`
- `nox -s test`