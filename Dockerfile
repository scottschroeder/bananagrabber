FROM rust:latest as build

# create a new empty shell project
RUN USER=root cargo new --bin bananagrabber
WORKDIR /bananagrabber

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/bananagrabber*
RUN cargo build --release

# our final base
FROM debian:buster-slim

RUN \
  apt-get update && \
  apt-get install -y ca-certificates && \
  apt-get clean

# copy the build artifact from the build stage
COPY --from=build /bananagrabber/target/release/bananagrabber .

# set the startup command to run your binary
CMD ["./bananagrabber"]
