#!/bin/bash

apt-get update && apt-get install -y \
    openssl libssl-dev \
    perl perl-modules libperl-dev build-essential \
    openssh-client \
    curl ca-certificates apt-transport-https \
    && rm -rf /var/lib/apt/lists/*

CODENAME=$(grep ^VERSION_CODENAME= /etc/os-release | cut -d= -f2 | tr -d '"')
CLANG_VERSION="17"
PROTOC_VERSION="33.1"

# install protoc
if [ ! -f /usr/local/bin/protoc ]
then
  apt-get update -qq && apt-get install -y unzip
  PROTOC_ZIP=protoc-${PROTOC_VERSION}-linux-x86_64.zip
  curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/${PROTOC_ZIP}
  unzip -o ${PROTOC_ZIP} -d /usr/local bin/protoc
  unzip -o ${PROTOC_ZIP} -d /usr/local 'include/*'
  rm -f ${PROTOC_ZIP}
fi

# install clang
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key -o /etc/apt/keyrings/llvm.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/llvm.asc] https://apt.llvm.org/${CODENAME}/ llvm-toolchain-${CODENAME}-$CLANG_VERSION main" | tee /etc/apt/sources.list.d/llvm-toolchain.list > /dev/null
apt-get -y update \
  && apt-get install -y clang-${CLANG_VERSION} lldb-${CLANG_VERSION} lld-${CLANG_VERSION} libclang-${CLANG_VERSION}-dev make m4 golang-go \
  && rm -rf /var/lib/apt/lists/*
update-alternatives --install /usr/bin/cc cc /usr/bin/clang-${CLANG_VERSION} 100
update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang-${CLANG_VERSION} 100
update-alternatives --install /usr/bin/ld ld /usr/bin/ld.lld-${CLANG_VERSION} 100

# install rust
RUST_VERSION=$(grep 'channel' 'rust-toolchain' | sed 's/.*= *"\([^"]*\)".*/\1/')
export RUSTUP_HOME="/opt/rustup_home"
export CARGO_HOME="/opt/cargo_home"
export CARGO_TARGET_DIR="/opt/cargo_target"
export PATH="$PATH:$CARGO_HOME/bin"
mkdir ${RUSTUP_HOME} ${CARGO_HOME} ${CARGO_TARGET_DIR}
curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path --default-toolchain ${RUST_VERSION} && \
  rustup component add rust-src --toolchain ${RUST_VERSION}-x86_64-unknown-linux-gnu
