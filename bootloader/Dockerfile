FROM ubuntu:20.10

ARG USER_ID=1000
ARG USER_GROUP=1000

RUN useradd -m -d /best_user best_user

RUN apt-get update
RUN apt-get install -y curl build-essential make parted mtools

RUN apt-get update
RUN apt-get install -y lld

RUN ln -s /usr/bin/lld-link-11 /usr/bin/lld-link

USER best_user

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/best_user/.cargo/bin:${PATH}"

RUN rustup default nightly
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup component add rust-src
