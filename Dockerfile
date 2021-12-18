# Adapted from: https://kerkour.com/rust-small-docker-image/#from-scratch
# Changed builder to use: https://github.com/emk/rust-musl-builder

####################################################################################################
## Builder
####################################################################################################
FROM clux/muslrust AS builder

# Create appuser
ENV USER=freebie-emailer
ENV UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /freebie-emailer

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /freebie-emailer

# Copy our build
COPY --from=builder /freebie-emailer/target/x86_64-unknown-linux-musl/release/freebie-emailer ./

# Use an unprivileged user.
USER freebie-emailer:freebie-emailer

CMD ["/freebie-emailer/freebie-emailer"]


####################################################################################################
## Labels
####################################################################################################

LABEL org.opencontainers.image.source https://github.com/urwrstkn8mare/freebie-emailer
