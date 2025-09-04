FROM rust:latest

# RUN apt-get update && apt-get install -y sudo

# RUN useradd -ms /bin/bash aelhadda && \
#     echo "aelhadda ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y make

ENTRYPOINT ["make"]