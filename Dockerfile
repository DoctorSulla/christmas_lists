FROM rust:latest

WORKDIR /app

RUN apt-get update

RUN apt install git curl vim build-essential -y

RUN git clone https://github.com/DoctorSulla/christmas_lists.git

WORKDIR /app/christmas_lists

RUN cargo build --release

ENTRYPOINT ["./target/release/christmas_lists"]
