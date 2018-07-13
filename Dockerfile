FROM ekidd/rust-musl-builder:nightly AS builder

ADD . ./

RUN sudo chown -R rust:rust /home/rust

RUN cargo update

RUN cargo build

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/debug/rustdag-server \
    /usr/local/bin/
CMD /usr/local/bin/rustdag-server
