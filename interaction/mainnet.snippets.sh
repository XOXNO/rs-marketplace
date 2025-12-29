EGLD=0x4d45582d373966303633
ADDRESS=erd1qqqqqqqqqqqqqpgq6wegs2xkypfpync8mn2sa5cmpqjlvrhwz5nqgepyg8
PROXY=https://gateway.xoxno.com

PROJECT="./output-docker/xoxno-protocol/xoxno-protocol.wasm"

SHARD2WrappingWEGLD=erd1qqqqqqqqqqqqqpgqmuk0q2saj0mgutxm4teywre6dl8wqf58xamqdrukln
AGGREGATOR=erd1qqqqqqqqqqqqqpgq5rf2sppxk2xu4m0pkmugw2es4gak3rgjah0sxvajva
ACCUMULATOR=erd1qqqqqqqqqqqqqpgq8538ku69p97lq4eug75y8d6g6yfwhd7c45qs4zvejt

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${OWNER} --gas-limit=125000000 --arguments 0x64 --send --outfile="deploy-mainnet.interaction.json" --proxy=${PROXY} --chain=1 || return
}

upgrade() {
    echo "Smart contract address: ${ADDRESS}"
    mxpy --verbose contract upgrade ${ADDRESS} --metadata-payable-by-sc --arguments ${ACCUMULATOR} ${AGGREGATOR} --bytecode=${PROJECT} --ledger \
    --gas-limit=250000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=1 || return
}

verifyContract() {
    mxpy --verbose contract verify "${ADDRESS}"  \
    --packaged-src=./output-docker/xoxno-protocol/xoxno-protocol-1.0.0.source.json --verifier-url="https://play-api.multiversx.com" \
    --docker-image="multiversx/sdk-rust-contract-builder:v10.0.0" --ledger  || return 
}

buildDocker() {
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v10.0.0"
}

removeRewardBalance() {
    mxpy --verbose contract call ${ADDRESS} --function="removeRewardBalance" --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0  --gas-limit=40000000 --send --proxy=${PROXY} --chain=1 || return
}