FROM rustlang/rust:nightly
EXPOSE 4200

WORKDIR /usr/src/server

COPY . .

RUN cargo install

CMD ["rustchain"]