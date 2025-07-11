FROM rust:1.82

WORKDIR /app
COPY . .

RUN touch op_logs.log

RUN cargo build --release

CMD ["./target/release/pensieve"]
