#FROM rust:1.80 as builder
#
#RUN apt update && mkdir /build
#
#COPY . /build
#
#RUN cd /build \
#    && cargo build --release


FROM debian:bullseye-slim

RUN apt update && apt install openssl ca-certificates -y

#COPY --from=builder /build/target/release/rust-imgbed /opt/rust-imgbed
COPY ./rust-imgbed /opt/rust-imgbed

WORKDIR /opt
EXPOSE 5800
CMD ["/opt/rust-imgbed"]