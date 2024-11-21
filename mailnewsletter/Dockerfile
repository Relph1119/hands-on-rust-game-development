FROM rust:latest AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:slim AS runtime

WORKDIR /app
COPY --from=builder /app/target/release/mailnewsletter mailnewsletter
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./mailnewsletter"]