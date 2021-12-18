FROM rust:1.57

LABEL org.opencontainers.image.source https://github.com/urwrstkn8mare/freebie-emailer

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["freebie-emailer"]
