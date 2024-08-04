# Start with a Rust base image
FROM rust:latest as builder

# Create a new empty shell project
RUN USER=root cargo new --bin thermite
WORKDIR /thermite

# Copy your manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# This trick will cache your dependencies as a separate Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy your source tree
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/thermite*
RUN cargo build --release

# Final base image
FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl

# Copy the build artifact from the build stage
COPY --from=builder /thermite/target/release/thermite .

# Set the necessary environment variables for runtime
ENV REDIS_URL=redis://localhost:6379
ENV TASKS_URL=0.0.0.0:8080

# Expose the port the server is listening on
EXPOSE 8080

# Run the binary
CMD ["./thermite"]