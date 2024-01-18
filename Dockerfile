FROM rust:latest as build

WORKDIR /app

RUN apt-get update

RUN apt install git curl vim build-essential -y

RUN git clone https://github.com/DoctorSulla/christmas_lists.git

WORKDIR /app/christmas_lists

RUN cargo build --release

FROM debian:stable

ENV APP_ENVIRONMENT PRODUCTION

WORKDIR /app/christmas_lists

RUN mkdir assets

COPY --from=build /app/christmas_lists/target/release/christmas_lists /app/christmas_lists

COPY --from=build /app/christmas_lists/assets/* /app/christmas_lists/assets/

ENTRYPOINT ["/app/christmas_lists/christmas_lists"]
