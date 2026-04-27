FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1
ENV PATH=/root/.cargo/bin:${PATH}

ENV APP_HOME=/workspace/qcel_howmany
WORKDIR ${APP_HOME}

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        python3 \
        python3-pip \
        python3-venv \
        build-essential \
        pkg-config \
        libssl-dev \
        ca-certificates \
        git \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain (stable) on Ubuntu base image.
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable \
    && rustc --version \
    && cargo --version

# Install uv and maturin for Python env + PyO3 build workflow.
RUN pip3 install --no-cache-dir uv && uv tool install maturin

RUN cargo install --locked typst-cli

# Show a welcome message for interactive bash sessions while keeping
# the system/default prompt behavior intact.
RUN printf '\n[ -n "$PS1" ] && echo "Welcome to qcel_howmany (${PWD}) — you can run: vi README.md to continue."\n' >> /etc/bash.bashrc

# Copy repo files.
COPY . ${APP_HOME}

# Install Python dependencies (including dev group) and build the extension.
RUN uv sync --group dev
RUN uv run maturin develop --release

# Keep shell as default entrypoint for interactive artifact use.
CMD ["/bin/bash"]
