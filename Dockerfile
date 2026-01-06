FROM --platform=linux/amd64 messense/rust-musl-cross:aarch64-musl as builder

WORKDIR /home/rust/src

COPY . .
RUN cargo build --release

FROM --platform=linux/arm64 alpine:latest

WORKDIR /app
RUN apk add --no-cache ca-certificates

COPY --from=builder /home/rust/src/target/aarch64-unknown-linux-musl/release/personal-website /app/server
COPY static /app/static

EXPOSE 3000
CMD ["./server"]
