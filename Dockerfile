ARG FEATURES=""

FROM docker.io/library/alpine:latest AS build
ARG FEATURES

RUN apk upgrade \
    && apk add --no-cache \
        binutils \
        build-base \
        ca-certificates \
        cmake \
        curl \
        linux-headers \
        perl

WORKDIR /app

RUN curl -sSf https://sh.rustup.rs \
    | sh -s -- --profile minimal --default-toolchain nightly --component rust-src -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY Cargo.lock Cargo.toml ./

RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build \
        --release \
        -Zbuild-std=std,panic_abort \
        --target="$(uname -m)-unknown-linux-musl" \
        --features="${FEATURES}"

COPY src ./src

RUN rm -f target/"$(uname -m)"-unknown-linux-musl/release/deps/roverse* \
    && cargo build \
        --release \
        -Zbuild-std=std,panic_abort \
        --target="$(uname -m)-unknown-linux-musl" \
        --features="${FEATURES}" \
    && cp target/"$(uname -m)"-unknown-linux-musl/release/roverse /roverse \
    && strip /roverse

FROM scratch

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /roverse /roverse

ENV ROVERSE_ADDR=0.0.0.0:8080
ENV ROVERSE_SECRET=

EXPOSE 8080
USER 65532:65532

CMD ["/roverse"]
