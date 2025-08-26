
FROM rust:1.89 as builder
WORKDIR /app
COPY . .
RUN cargo build --bin forge_of_stories_server --features server --release

FROM ubuntu:24.04
WORKDIR /app
COPY --from=builder /app/target/release/forge_of_stories_server .
# Ports frei geben, z.B. für TUI/CLI, Server und Web-Interface
EXPOSE 8000
EXPOSE 8080

CMD ["./forge_of_stories_server"]
