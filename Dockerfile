# Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.

FROM docker.io/rust:1.89-alpine3.22 AS build
RUN set -e -x; \
    apk update; \
    apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.* ./
COPY src ./src
RUN set -e -x; cargo build --release

FROM docker.io/alpine:3.22
RUN set -e -x; \
    apk update; \
    apk add --no-cache git; \
    rm -rf /var/cache/apk/*
RUN set -e -x; \
    git config --system safe.directory '*'
COPY --from=build /src/target/release/ghaction-version-gen /usr/local/bin/
CMD ["/usr/local/bin/ghaction-version-gen"]
