FROM golang:1.22 AS builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o atlas-gateway main.go

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/atlas-gateway /app/atlas-gateway
EXPOSE 8080
CMD ["/app/atlas-gateway"]
