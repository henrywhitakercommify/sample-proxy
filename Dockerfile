FROM rust:1.95.0-alpine AS build

WORKDIR /build
COPY ./ /build
RUN cd /build && cargo build --release

FROM alpine:3.23.4 AS certs

RUN apk add ca-certificates

FROM alpine:3.23.4

COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /build/target/release/sample-proxy /sample-proxy

CMD ["/sample-proxy"]
