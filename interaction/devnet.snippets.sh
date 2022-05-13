
ALICE="${USERS}/alice.pem"  
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
ADDRESS=erd1qqqqqqqqqqqqqpgqn4fnwl43hhchz9emdy66eh5azzhl599zd8ssxjdyh3
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.elrond.com

deploy() {
    echo ${PROJECT}
    erdpy --verbose contract deploy --project=${PROJECT} --metadata-payable --recall-nonce --pem=${ALICE} --gas-limit=225000000 --arguments 0xFA --send --outfile="deploy-devnet.interaction.json" --proxy=${PROXY} --chain=D || return

    TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-devnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    echo "Smart contract address: ${ADDRESS}"
    erdpy --verbose contract upgrade ${ADDRESS} --arguments 0x64 --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=250000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain="D" || return
}

getDustAmountLeft() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=6500000 --function="getDustAmountLeft" --arguments 0x45474c442d633365323066 0x00 --send --proxy=${PROXY} --chain=D
}

setStatusOn() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=15000000 --function="setStatus" --arguments 0x01 --send --proxy=${PROXY} --chain=D
}

setStatusOff() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=6500000 --function="setStatus" --arguments 0x00 --send --proxy=${PROXY} --chain=D
}

setAcceptedTokens() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=10500000 --function="setAcceptedTokens" --arguments 0x45474c44 --send --proxy=${PROXY} --chain=D
}
removeAcceptedTokens() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=6500000 --function="removeAcceptedTokens" --arguments 0x4c4b4d45582d3830356538 --send --proxy=${PROXY} --chain=D
}
withdraw() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="withdraw" --arguments 0x05 --send --proxy=${PROXY} --chain=D
}

endAuction() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="endAuction" --arguments 0x05 --send --proxy=${PROXY} --chain=D
}

bidCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

bidBob() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=1000000000000000000 --send --proxy=${PROXY} --chain=D
}

bidEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

bidESDTEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=D
}

bidESDTBob() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785d8a0000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=D
}

bidESDTCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=D
}
buyEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

buyCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

buyESDTEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTNFTTransfer" --arguments 0x05 0x45474c442d633365323066 0x01 --value=100000000000000000 --send --proxy=${PROXY} --chain=D
}

buyESDTCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785D8A0000 0x627579 0x01 0x4b42422d316339353733 0x01 --send --proxy=${PROXY} --chain=D
}

listNFT() {
    erdpy --verbose contract call ${CAROLWALLET} --recall-nonce --pem=${CAROL} --gas-limit=35000000 --function="ESDTNFTTransfer" \
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
    erdpy --verbose contract query ${ADDRESS} --function="getListingsCount" --proxy=${PROXY}
}

getListings() {
    erdpy --verbose contract query ${ADDRESS} --function="getListings" --proxy=${PROXY}
}

getStatus() {
    erdpy --verbose contract query ${ADDRESS} --function="getStatus" --proxy=${PROXY}
}

getLastValidAuctionId() {
    erdpy --verbose contract query ${ADDRESS} --function="getLastValidAuctionId" --proxy=${PROXY}
}

getAcceptedTokens() {
    erdpy --verbose contract query ${ADDRESS} --function="getAcceptedTokens" --proxy=${PROXY}
}

getMarketplaceCutPercentage() {
    erdpy --verbose contract query ${ADDRESS} --function="getMarketplaceCutPercentage" --proxy=${PROXY}
}

getListingsByWallet() {
    erdpy --verbose contract query ${ADDRESS} --function="getListingsByWallet" --arguments ${EVEHEX} --proxy=${PROXY}
}

getActiveListingsBids() {
    erdpy --verbose contract query ${ADDRESS} --function="getActiveListingsBids" --arguments ${BOBHEX} --proxy=${PROXY}
}

getTokenBalanceDifference() {
    erdpy --verbose contract query ${ADDRESS} --function="getTokenBalanceDifference" --arguments 0x45474c44 0x00 --proxy=${PROXY}
}

getTokenItemsForSale() {
    erdpy --verbose contract query ${ADDRESS} --function="getTokenItemsForSale" --arguments 0x4b42422d316339353733 --proxy=${PROXY}
}

getTokenItemsQuantityForSale() {
    erdpy --verbose contract query ${ADDRESS} --function="getTokenItemsQuantityForSale" --arguments 0x4b42422d316339353733 0x05 --proxy=${PROXY}
}

getTokenItemsForSaleCount() {
    erdpy --verbose contract query ${ADDRESS} --function="getTokenItemsForSaleCount" --arguments 0x4b42422d316339353733 --proxy=${PROXY}
}

doesAuctionExist() {
    erdpy --verbose contract query ${ADDRESS} --function="doesAuctionExist" --arguments 0x05 --proxy=${PROXY}
}

getAuctionedToken() {
    erdpy --verbose contract query ${ADDRESS} --function="getAuctionedToken" --arguments 0x05 --proxy=${PROXY}
}

getAuctionType() {
    erdpy --verbose contract query ${ADDRESS} --function="getAuctionType" --arguments 0x05 --proxy=${PROXY}
}

getPaymentTokenForAuction() {
    erdpy --verbose contract query ${ADDRESS} --function="getPaymentTokenForAuction" --arguments 0x05 --proxy=${PROXY}
}

getMinMaxBid() {
    erdpy --verbose contract query ${ADDRESS} --function="getMinMaxBid" --arguments 0x05 --proxy=${PROXY}
}

getStartTime() {
    erdpy --verbose contract query ${ADDRESS} --function="getStartTime" --arguments 0x05 --proxy=${PROXY}
}

getDeadline() {
    erdpy --verbose contract query ${ADDRESS} --function="getDeadline" --arguments 0x05 --proxy=${PROXY}
}

getOriginalOwner() {
    erdpy --verbose contract query ${ADDRESS} --function="getOriginalOwner" --arguments 0x05 --proxy=${PROXY}
}

getCurrentWinningBid() {
    erdpy --verbose contract query ${ADDRESS} --function="getCurrentWinningBid" --arguments 0x05 --proxy=${PROXY}
}

getCurrentWinner() {
    erdpy --verbose contract query ${ADDRESS} --function="getCurrentWinner" --arguments 0x05 --proxy=${PROXY}
}

getFullAuctionData() {
    erdpy --verbose contract query ${ADDRESS} --function="getFullAuctionData" --arguments 0x05 --proxy=${PROXY}
}