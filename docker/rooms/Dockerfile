FROM alpine
RUN apk add alpine-sdk cmake openssl openssl-dev curl postgresql-client
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="${PATH}:/root/.cargo/bin"
RUN cargo install sqlx-cli
COPY . /rooms
WORKDIR /rooms
RUN cargo build
CMD sqlx migrate run && cargo run
