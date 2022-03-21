#!/bin/bash


if [ -z `which rustup` ]; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ./install_rust.sh
    sh ./install_rust.sh -y
fi


rustup target install wasm32-unknown-unknown

cargo install toml-cli

WASM_BINDGEN_VERSION=`toml get ./Cargo.toml 'target."cfg(target_arch = \"wasm32\")".dependencies.wasm-bindgen.version' | xargs echo`

# See: https://github.com/rustwasm/wasm-bindgen/issues/2841
# cargo install wasm-bindgen-cli --vers $WASM_BINDGEN_VERSION
cargo install --git https://github.com/cdata/wasm-bindgen wasm-bindgen-cli

if [ -z ${BROWSERSTACK+x} ]; then
    cargo install webdriver-install
    webdriver_install --install chromedriver

    CHROMEDRIVER=~/.webdrivers/chromedriver \
        cargo test --target wasm32-unknown-unknown --features web
else

    set -x

    if [ -z `which jq` ]; then
        sudo apt install jq
    fi

    BROWSERSTACK_SESSION="{
    \"build\": \"$BROWSERSTACK_BUILD_NAME\",
    \"project\": \"$BROWSERSTACK_PROJECT_NAME\",
    \"browserstack.local\": \"true\",
    \"browserstack.localIdentifier\": \"$BROWSERSTACK_LOCAL_IDENTIFIER\",
    \"browserstack.user\": \"$BROWSERSTACK_USERNAME\",
    \"browserstack.key\": \"$BROWSERSTACK_ACCESS_KEY\"
}"

    # TODO: Locate webdriver.json relative to script being invoked..
    WEBDRIVER="`cat ./webdriver.json`"
    echo $WEBDRIVER | jq ". + $BROWSERSTACK_SESSION" > webdriver.json

    cargo build --target wasm32-unknown-unknown --features web
    CHROMEDRIVER_REMOTE=https://hub-cloud.browserstack.com/wd/hub/ \
        cargo test --target wasm32-unknown-unknown --features web

    set +x
fi
