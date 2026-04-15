FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y libclang-dev pkg-config
RUN cargo build --release

FROM ubuntu:24.04
COPY --from=builder /app/target/release/pint /bin/pint
ENTRYPOINT ["/bin/pint"]

