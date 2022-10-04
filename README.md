## Todo
- [ ] Add type annotations https://maturin.rs/project_layout.html
- [ ] Distribute for multiple python versions https://pyo3.rs/v0.17.1/building_and_distribution.html
- [ ] Figure out GIL https://pyo3.rs/v0.17.1/types.html
- [ ] Add error handling
- [ ] Add mmdb as a submodule

## Develop

- `git clone --recurse-submodules git@github.com:andrusha/pandas-maxminddb.git`
- `asdf install`
- `python -m venv .venv`
- `source .venv/bin/activate`
- `pip install .`
- `pip install nox`
- `wget https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-City.mmdb`
- `nox -s test`