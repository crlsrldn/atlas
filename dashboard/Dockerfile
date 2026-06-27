FROM denoland/deno:alpine
WORKDIR /app
COPY . .
RUN deno task build
EXPOSE 8000
CMD ["task", "preview"]
