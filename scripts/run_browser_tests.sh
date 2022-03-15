#!/bin/bash

cargo install webdriver-install

webdriver_install --install chromedriver

CHROMEDRIVER=~/.webdrivers/chromedriver \
    cargo test --target wasm32-unknown-unknown
