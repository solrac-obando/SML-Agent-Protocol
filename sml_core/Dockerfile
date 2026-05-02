FROM rust:1.75-bookworm

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN curl -sL https://get.ollama.ai | sh

WORKDIR /workspace/sml

COPY Cargo.toml ./
COPY src ./src
COPY benches ./benches
COPY grammar ./grammar
COPY tests ./tests

RUN cargo build --release --locked 2>&1 | tail -20

RUN echo '#!/bin/bash\n\
echo "SML Protocol Dispatcher v0.1.0"\n\
echo "================================"\n\
echo ""\n\
if [ "$1" = "--help" ]; then\n\
    echo "Usage: sml [COMMAND] [OPTIONS]"\n\
    echo ""\n\
    echo "Commands:"\n\
    echo "  start         Start the SML dispatcher in interactive mode"\n\
    echo "  benchmark     Run performance benchmarks"\n\
    echo "  test          Run parser tests"\n\
    echo "  grammar       Display GBNF grammar"\n\
    echo "  server [PORT] Start HTTP server (default: 8080)"\n\
    echo "  ollama [MODEL] Start Ollama chat mode"\n\
    echo ""\n\
    echo "Examples:"\n\
    echo "  sml start"\n\
    echo "  sml benchmark"\n\
    echo "  sml server 8080"\n\
    echo "  sml execute @[read:Cargo.toml]"\n\
    exit 0\n\
fi\n\
case "$1" in\n\
    start)     exec /workspace/sml/target/release/sml_core ;;\n\
    benchmark) exec /workspace/sml/target/release/sml_core --benchmark ;;\n\
    test)      exec /workspace/sml/target/release/sml_core --test-parser ;;\n\
    grammar)   cat /workspace/sml/grammar/gbnf.sml ;;\n\
    execute)   exec /workspace/sml/target/release/sml_core --execute "$2" ;;\n\
    server)    exec /workspace/sml/target/release/sml_core --server "${2:-8080}" ;;\n\
    ollama)    exec /workspace/sml/target/release/sml_core --ollama "${2:-qwen2.5-coder:3b}" ;;\n\
    *)         exec /workspace/sml/target/release/sml_core ;;\n\
esac' > /usr/local/bin/sml && chmod +x /usr/local/bin/sml

RUN echo '#!/bin/bash\n\
echo "Starting Ollama..."\n\
ollama serve &' > /entrypoint.sh && chmod +x /entrypoint.sh

ENV PATH="/usr/local/bin:${PATH}"
ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["/entrypoint.sh"]
CMD ["sml", "server"]