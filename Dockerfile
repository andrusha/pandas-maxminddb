FROM python:3.8-buster

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile default --default-toolchain stable
RUN pip install nox maturin pandas numpy
ENV PATH=/root/.cargo/env:$PATH

WORKDIR /code
