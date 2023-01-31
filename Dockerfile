FROM rust:bullseye as builder
WORKDIR /usr/src/app
COPY . .
RUN apt update && apt install protobuf-compiler libprotobuf-dev -y \
    && cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/terminusdb-grpc-labelstore-server /app/
CMD ["./terminusdb-grpc-labelstore-server"]
