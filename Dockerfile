FROM ekidd/rust-musl-builder:nightly AS builder

# Load dependencies first, to allow caching when only code changes
ADD Cargo.* ./
ADD lib/Cargo.toml lib/
ADD server/Cargo.toml server/
ADD client/Cargo.toml client/

ADD lib/src/lib.rs lib/src/
ADD server/src/main.rs server/src/
ADD client/src/main.rs client/src/

RUN sudo chown -R rust:rust /home/rust
RUN cargo fetch

# Load all the code
ADD . ./

RUN sudo chown -R rust:rust /home/rust

RUN cargo build

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/debug/rustdag-server \
    /usr/local/bin/
EXPOSE 8000
ENV ROCKET_ADDRESS 0.0.0.0
CMD /usr/local/bin/rustdag-server
