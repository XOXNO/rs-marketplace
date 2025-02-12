
ADDRESS=erd1qqqqqqqqqqqqqpgql0dnz6n5hpuw8cptlt00khd0nn4ja8eadsfq2xrqw4
PROXY=https://devnet-gateway.xoxno.com
SHARD2WrappingWEGLD=erd1qqqqqqqqqqqqqpgqvn9ew0wwn7a3pk053ezex98497hd4exqdg0q8v2e0c
XOXNOPairSwap=erd1qqqqqqqqqqqqqpgqae44n6t0fhg40zmtq3lzjk58f8t7envn0n4sj7x6pl
ASHSWAP=erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6
ACCUMULATOR=erd1qqqqqqqqqqqqqpgqyxfc4r5fmw2ljcgwxj2nuzv72y9ryvyhah0sgn5vv2

PROJECT="./output-docker/xoxno-protocol/xoxno-protocol.wasm"

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --metadata-payable-by-sc --arguments 0x64 "erd1cfyadenn4k9wndha0ljhlsdrww9k0jqafqq626hu9zt79urzvzasalgycz" ${SHARD2WrappingWEGLD} str:WEGLD-a28c59 ${ASHSWAP} \
    --bytecode="/Users/truststaking/Documents/GitHub/marketplace/output/xoxno-protocol.wasm" --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=2 \
    --gas-limit=525000000 --send --proxy=${PROXY} --chain=D || return
}

upgrade() {
    echo "Smart contract address: ${ADDRESS}"
    mxpy contract upgrade ${ADDRESS} --metadata-payable-by-sc --arguments ${ACCUMULATOR} ${ASHSWAP} --bytecode=${PROJECT} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=2 \
    --gas-limit=350000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain="D" || return
}

verifyContract() {
    mxpy --verbose contract verify "${ADDRESS}"  \
    --packaged-src=./output-docker/xoxno-protocol/xoxno-protocol-1.0.0.source.json --verifier-url="https://devnet-play-api.multiversx.com" \
    --docker-image="multiversx/sdk-rust-contract-builder:v8.0.1" --ledger --ledger-account-index=0 --ledger-address-index=2  || return 
}

buildDocker() {
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v8.0.1"
}