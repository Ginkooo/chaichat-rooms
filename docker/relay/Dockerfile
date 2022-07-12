FROM alpine
RUN apk add alpine-sdk cmake rust cargo
RUN git clone https://github.com/libp2p/rust-libp2p
WORKDIR rust-libp2p
RUN cargo build --example relay_v2 -p libp2p-relay
CMD target/debug/examples/relay_v2 --port 4001 --secret-key-seed 0
