FROM rustlang/rust:nightly
EXPOSE 4200

WORKDIR /usr/src/server

COPY . .

RUN cargo update

RUN cargo install --path server

CMD ["rustdag-server"]