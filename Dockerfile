FROM rust:1.57

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["freebie-emailer"]
