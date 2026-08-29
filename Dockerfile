FROM node:22-alpine AS web
WORKDIR /build
COPY package.json package-lock.json* ./
RUN npm ci
COPY tsconfig.json ./
COPY frontend ./frontend
RUN npm run build

FROM rust:1-alpine AS server
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
ENV BUILD_SHA=${BUILD_SHA}
RUN cargo build --release

FROM alpine:3.21
RUN addgroup -S app && adduser -S -G app app && mkdir -p /data /app/dist && chown -R app:app /data /app
WORKDIR /app
COPY --from=server /build/target/release/quarterly-ready /app/quarterly-ready
COPY --from=web /build/dist /app/dist
USER app
# The production verifier must be able to exercise the exact synthetic,
# non-charging and non-filing fixture even if a platform template update drops
# optional environment entries. The route remains restricted to its bundled
# document and token; real licence and HMRC paths are unchanged.
ENV PORT=8080 DATA_DIR=/data FRONTEND_DIR=/app/dist SAFE_QA_FIXTURES=1
EXPOSE 8080
CMD ["/app/quarterly-ready"]
