FROM rust:latest

RUN apt-get update && apt-get install -y sudo

RUN useradd -ms /bin/bash aelhadda && \
    echo "aelhadda ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

WORKDIR /app
COPY . .

RUN cargo build --release && \
    cp target/release/shell /usr/local/bin/0-shell && \
    echo "/usr/local/bin/0-shell" >> /etc/shells && \
    chsh -s /usr/local/bin/0-shell aelhadda

USER aelhadda
WORKDIR /home/aelhadda

CMD ["/usr/local/bin/0-shell"]