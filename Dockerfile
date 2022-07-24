# Multistage docker build, requires docker 17.05

# builder stage
FROM nvidia/cuda:10.0-devel as builder

RUN set -ex && \
    apt-get update && \
    apt-get --no-install-recommends --yes install \
        libncurses5-dev \
        libncursesw5-dev \
        cmake \
        git \
        curl \
        libssl-dev \
        pkg-config

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN git clone https://github.com/mugleproject/mugle-miner && cd mugle-miner && git submodule update --init

RUN cd mugle-miner && sed -i '/^cuckoo_miner = {/s/^/#/' Cargo.toml && sed -i '/^#.*build-cuda-plugins"]/s/^#//' Cargo.toml

RUN cd mugle-miner && $HOME/.cargo/bin/cargo build --release

# runtime stage
FROM nvidia/cuda:10.0-base

RUN set -ex && \
    apt-get update && \
    apt-get --no-install-recommends --yes install \
    libncurses5 \
    libncursesw5

COPY --from=builder /mugle-miner/target/release/mugle-miner /mugle-miner/target/release/mugle-miner
COPY --from=builder /mugle-miner/target/release/plugins/* /mugle-miner/target/release/plugins/
COPY --from=builder /mugle-miner/mugle-miner.toml /mugle-miner/mugle-miner.toml

WORKDIR /mugle-miner

RUN sed -i -e 's/run_tui = true/run_tui = false/' mugle-miner.toml

RUN echo '#!/bin/bash\n\
if [ $# -eq 1 ]\n\
   then\n\
sed -i -e 's/127.0.0.1/\$1/g' mugle-miner.toml\n\
fi\n\
./target/release/mugle-miner' > run.sh

# If the mugle server is not at 127.0.0.1 provide the ip or hostname to the container
# by command line (i.e. docker run --name miner1 --rm -i -t miner_image 1.2.3.4)

ENTRYPOINT ["sh", "run.sh"]
