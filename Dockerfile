FROM rust:1.57

WORKDIR /usr/src/app
COPY ./app .

RUN cargo install --path .

CMD ["freebie-emailer"]
