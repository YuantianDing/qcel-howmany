FROM rust:1-slim-trixie AS builder

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

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
    && rm -rf /var/lib/apt/lists/*

RUN pip3 install --break-system-packages --no-cache-dir maturin
RUN cargo install --locked typst-cli

COPY python ${APP_HOME}/python
COPY src ${APP_HOME}/src
COPY Cargo.toml ${APP_HOME}/
COPY Cargo.lock ${APP_HOME}/
COPY pyproject.toml ${APP_HOME}/
COPY uv.lock ${APP_HOME}/

# Build wheel from the Rust/Python project.
RUN maturin build --release --interpreter python3 --out /tmp/wheels \
    && rm -rf /root/.cache/pip /root/.cargo/registry /root/.cargo/git



FROM rust:1-slim-trixie

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

ENV APP_HOME=/workspace/qcel_howmany
WORKDIR ${APP_HOME}

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        python3 \
        python3-pip \
        python3-venv \
        ca-certificates \
        bash \
    && rm -rf /var/lib/apt/lists/*

# Install the built wheel into the runtime image.
COPY --from=builder /tmp/wheels /tmp/wheels
RUN pip3 install --break-system-packages --no-cache-dir /tmp/wheels/*.whl \
    && pip3 install --break-system-packages --no-cache-dir \
        pytest \
        pandas \
        pillow \
        pylatexenc \
        tables \
        tqdm \
    && rm -rf /tmp/wheels /root/.cache/pip

COPY --from=builder /usr/local/cargo/bin/typst /usr/local/bin/typst

# Keep repository content available for scripts, data, and experiment runners.
COPY quartz ${APP_HOME}/quartz
COPY scripts ${APP_HOME}/scripts
COPY python ${APP_HOME}/python
COPY src ${APP_HOME}/src
COPY Cargo.toml ${APP_HOME}/
COPY pyproject.toml ${APP_HOME}/
COPY uv.lock ${APP_HOME}/


COPY README.md ${APP_HOME}/
# Show a welcome message for interactive bash sessions while keeping
# the system/default prompt behavior intact.
RUN printf '%s\n' \
    '' \
    '   ___    ____ _____ _     ' \
    '  / _ \  / ___| ____| |    ' \
    ' | | | || |   |  _| | |    ' \
    ' | |_| || |___| |___| |___ ' \
    '  \__\_\ \____|_____|_____|' \
    '' \
    'Welcome to qcel_howmany.' \
    'See `README.md` for more details.' \
    '' \
    > /etc/motd.qcel
RUN printf '\n[ -n "$PS1" ] && cat /etc/motd.qcel\n' >> /etc/bash.bashrc

# Keep shell as default entrypoint for interactive artifact use.
CMD ["/bin/bash"]
