FROM python:3 AS scraper

WORKDIR /app
COPY ./fetch_tags.py .

# hadolint ignore=DL3013
RUN pip install --no-cache-dir pipenv
RUN pipenv install

# hadolint ignore=DL3059
RUN pipenv run python ./fetch_tags.py

FROM rust:slim AS builder
# hadolint ignore=DL3008
RUN apt-get update && apt-get -y --no-install-recommends install libssl-dev pkg-config

WORKDIR /usr/src/risu
COPY . .

RUN cargo fetch

# hadolint ignore=DL3059
RUN cargo install --path .

FROM rust:slim

WORKDIR /app

COPY --from=scraper /app/data /app/data
COPY --from=builder /usr/local/cargo/bin/risu /app/risu
CMD ["/app/risu"]