# Multi-Stage Build: nur das fertige Binary landet im finalen Container
FROM rust:1.89 as builder
WORKDIR /app
COPY . .
RUN cargo build --bin forge_of_stories_server --features server --release

FROM ubuntu:24.04
WORKDIR /app
COPY --from=builder /app/target/release/forge_of_stories_server .
# Startet den Server beim Container-Start
CMD ["./forge_of_stories_server"]
