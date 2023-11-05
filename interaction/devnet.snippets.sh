
ALICE="/Users/truststaking/Desktop/PEM/alice.pem"
ALICEWALET=erd1qyu5wthldzr8wx5c9ucg8kjagg0jfs53s8nr3zpz3hypefsdd8ssycr6th
ALICEHEX=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1
BOB="${USERS}/bob.pem"
BOBWALLET=erd1spyavw0956vq68xj8y4tenjpq2wd5a9p2c6j8gsz7ztyrnpxrruqzu66jx
BOBHEX=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8
CAROL="${USERS}/carol.pem"
CAROLWALLET=erd1k2s324ww2g0yj38qn2ch2jwctdy8mnfxep94q9arncc6xecg3xaq6mjse8
CAROLHEX=0xb2a11555ce521e4944e09ab17549d85b487dcd26c84b5017a39e31a3670889ba
EVE="${USERS}/eve.pem"
EVEWALLET=erd18tudnj2z8vjh0339yu3vrkgzz2jpz8mjq0uhgnmklnap6z33qqeszq2yn4
EVEHEX=0x3af8d9c9423b2577c6252722c1d90212a4111f7203f9744f76fcfa1d0a310033
SC=0x000000000000000005008c2c42c102c9b6c3d2422e522cdf7b903e6ae78a69e1
EGLD=0x4d45582d373966303633 #45474c44 2d633365323066
ADDRESS=erd1qqqqqqqqqqqqqpgql0dnz6n5hpuw8cptlt00khd0nn4ja8eadsfq2xrqw4
DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.multiversx.com
SHARD2WrappingWEGLD=erd1qqqqqqqqqqqqqpgqvn9ew0wwn7a3pk053ezex98497hd4exqdg0q8v2e0c
XOXNOPairSwap=erd1qqqqqqqqqqqqqpgqae44n6t0fhg40zmtq3lzjk58f8t7envn0n4sj7x6pl

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --metadata-payable-by-sc --arguments 0x64 "erd1cfyadenn4k9wndha0ljhlsdrww9k0jqafqq626hu9zt79urzvzasalgycz" ${SHARD2WrappingWEGLD} str:WEGLD-a28c59 "erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6" --bytecode="/Users/truststaking/Documents/GitHub/marketplace/output/xoxno-protocol.wasm" --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=2 --gas-limit=525000000 --send --proxy=${PROXY} --chain=D || return

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

# ${XOXNOPairSwap} str:XOXNO-2d9386
upgrade() {
    echo "Smart contract address: ${ADDRESS}"
    mxpy --verbose contract upgrade ${ADDRESS} --metadata-payable-by-sc --arguments 0x64 "erd1cfyadenn4k9wndha0ljhlsdrww9k0jqafqq626hu9zt79urzvzasalgycz" ${SHARD2WrappingWEGLD} str:WEGLD-a28c59 "erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6" --bytecode="/Users/truststaking/Documents/GitHub/marketplace/output/xoxno-protocol.wasm" --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=2 \
    --gas-limit=350000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain="D" || return
}


setExtraFees() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=15000000 --function="setExtraFees" --arguments "str:MICE-f17a5e" 0x03E8 erd1fmd662htrgt07xxd8me09newa9s0euzvpz3wp0c4pz78f83grt9qm6pn57 --send --proxy=${PROXY} --chain=D
}

getDustAmountLeft() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=6500000 --function="getDustAmountLeft" --arguments 0x45474c442d633365323066 0x00 --send --proxy=${PROXY} --chain=D
}

setStatusOn() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=2 --gas-limit=15000000 --function="setStatus" --arguments 0x01 --send --proxy=${PROXY} --chain=D
}

addWitelistedSC() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=100000000 --function="addWitelistedSC" --arguments erd1qqqqqqqqqqqqqpgqn8qgd8redqt0lx6sn2p0rwcmvals870t03as9g0m9h --send --proxy=${PROXY} --chain=D
}

setStatusOff() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=6500000 --function="setStatus" --arguments 0x00 --send --proxy=${PROXY} --chain=D
}

setAcceptedTokens() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=2 --gas-limit=10500000 --function="setAcceptedTokens" --arguments str:WEGLD-a28c59 --send --proxy=${PROXY} --chain=D
}
removeAcceptedTokens() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=6500000 --function="removeAcceptedTokens" --arguments 0x4c4b4d45582d3830356538 --send --proxy=${PROXY} --chain=D
}
withdraw() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="withdraw" --arguments 0x05 --send --proxy=${PROXY} --chain=D
}

endAuction() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="endAuction" --arguments 0x05 --send --proxy=${PROXY} --chain=D
}

bidCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

bidBob() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=1000000000000000000 --send --proxy=${PROXY} --chain=D
}

bidEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

bidESDTEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=D
}

bidESDTBob() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785d8a0000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=D
}

bidESDTCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=D
}
buyEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

buyCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

buyESDTEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTNFTTransfer" --arguments 0x05 0x45474c442d633365323066 0x01 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

buyESDTCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785D8A0000 0x627579 0x01 0x4b42422d316339353733 0x01 --send --proxy=${PROXY} --chain=D
}

listNFT() {
    mxpy --verbose contract call ${CAROLWALLET} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="ESDTNFTTransfer" \
    --arguments \
    0x4b42422d316339353733 \
    0x05 \
    0x01 \
    ${SC} \
    0x6c697374696e67 \
    0x002386F26FC10000 \
    0x016345785D8A0000 \
    0x61A80C80 \
    ${EGLD} \
    0x01 \
    --send --proxy=${PROXY} --chain=D
}

getListingsCount() {
    mxpy --verbose contract query ${ADDRESS} --function="getListingsCount" --proxy=${PROXY}
}

getListings() {
    mxpy --verbose contract query ${ADDRESS} --function="getListings" --proxy=${PROXY}
}

getStatus() {
    mxpy --verbose contract query ${ADDRESS} --function="getStatus" --proxy=${PROXY}
}

getLastValidAuctionId() {
    mxpy --verbose contract query ${ADDRESS} --function="getLastValidAuctionId" --proxy=${PROXY}
}

getAcceptedTokens() {
    mxpy --verbose contract query ${ADDRESS} --function="getAcceptedTokens" --proxy=${PROXY}
}

getMarketplaceCutPercentage() {
    mxpy --verbose contract query ${ADDRESS} --function="getMarketplaceCutPercentage" --proxy=${PROXY}
}

getListingsByWallet() {
    mxpy --verbose contract query ${ADDRESS} --function="getListingsByWallet" --arguments ${EVEHEX} --proxy=${PROXY}
}

getActiveListingsBids() {
    mxpy --verbose contract query ${ADDRESS} --function="getActiveListingsBids" --arguments ${BOBHEX} --proxy=${PROXY}
}

getTokenBalanceDifference() {
    mxpy --verbose contract query ${ADDRESS} --function="getTokenBalanceDifference" --arguments 0x45474c44 0x00 --proxy=${PROXY}
}

getTokenItemsForSale() {
    mxpy --verbose contract query ${ADDRESS} --function="getTokenItemsForSale" --arguments str:COMPANIONS-71cb7c --proxy=${PROXY}
}

getTokenItemsQuantityForSale() {
    mxpy --verbose contract query ${ADDRESS} --function="getTokenItemsQuantityForSale" --arguments str:COMPANIONS-71cb7c 15 --proxy=${PROXY}
}

getTokenItemsForSaleCount() {
    mxpy --verbose contract query ${ADDRESS} --function="getTokenItemsForSaleCount" --arguments str:COMPANIONS-71cb7c --proxy=${PROXY}
}

doesAuctionExist() {
    mxpy --verbose contract query ${ADDRESS} --function="doesAuctionExist" --arguments 0x05 --proxy=${PROXY}
}

getAuctionedToken() {
    mxpy --verbose contract query ${ADDRESS} --function="getAuctionedToken" --arguments 0x05 --proxy=${PROXY}
}

getAuctionType() {
    mxpy --verbose contract query ${ADDRESS} --function="getAuctionType" --arguments 0x05 --proxy=${PROXY}
}

getPaymentTokenForAuction() {
    mxpy --verbose contract query ${ADDRESS} --function="getPaymentTokenForAuction" --arguments 0x05 --proxy=${PROXY}
}

getMinMaxBid() {
    mxpy --verbose contract query ${ADDRESS} --function="getMinMaxBid" --arguments 0x05 --proxy=${PROXY}
}

getStartTime() {
    mxpy --verbose contract query ${ADDRESS} --function="getStartTime" --arguments 0x05 --proxy=${PROXY}
}

getDeadline() {
    mxpy --verbose contract query ${ADDRESS} --function="getDeadline" --arguments 0x05 --proxy=${PROXY}
}

getOriginalOwner() {
    mxpy --verbose contract query ${ADDRESS} --function="getOriginalOwner" --arguments 0x05 --proxy=${PROXY}
}

getCurrentWinningBid() {
    mxpy --verbose contract query ${ADDRESS} --function="getCurrentWinningBid" --arguments 0x05 --proxy=${PROXY}
}

getCurrentWinner() {
    mxpy --verbose contract query ${ADDRESS} --function="getCurrentWinner" --arguments 0x05 --proxy=${PROXY}
}

getFullAuctionData() {
    mxpy --verbose contract query ${ADDRESS} --function="getFullAuctionData" --arguments 0x05 --proxy=${PROXY}
}

getLastValidGlobalOfferId() {
    mxpy --verbose contract query ${ADDRESS} --function="getLastValidGlobalOfferId" --proxy=${PROXY}
}
getGlobalOffers() {
    mxpy --verbose contract query ${ADDRESS} --function="getGlobalOffers" --proxy=${PROXY}
}