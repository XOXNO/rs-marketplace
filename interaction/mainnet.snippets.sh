
   
ALICE="${USERS}/alice.pem"  
OWNER="${USERS}/marketplace.pem"  
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
ADDRESS=erd1qqqqqqqqqqqqqpgq6wegs2xkypfpync8mn2sa5cmpqjlvrhwz5nqgepyg8
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-mainnet)
PROXY=https://gateway.elrond.com

deploy() {
    echo ${PROJECT}
    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${OWNER} --gas-limit=125000000 --arguments 0x64 --send --outfile="deploy-mainnet.interaction.json" --proxy=${PROXY} --chain=1 || return

    TRANSACTION=$(erdpy data parse --file="deploy-mainnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-mainnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-mainnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-mainnet --value=${TRANSACTION}

    echo ""
}

upgrade() {
    echo "Smart contract address: ${ADDRESS}"
    erdpy --verbose contract upgrade ${ADDRESS} --arguments 0x64 --project=${PROJECT} --recall-nonce --pem=${OWNER} \
    --gas-limit=160000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=1 || return
}

addWitelistedSC() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="addWitelistedSC" --arguments 0x00000000000000000500f237daf1b2cde3b77015feede76308bbd7999b9a2328 --send --proxy=${PROXY} --chain=1
}

setStatusOn() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="setStatus" --arguments 0x01 --send --proxy=${PROXY} --chain=1
}

setRewardTicker() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="setRewardTicker" --arguments 0x57415445522d396564343030 --send --proxy=${PROXY} --chain=1
}

setDefaultRewardAmount() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="setDefaultRewardAmount" --arguments 0x8AC7230489E80000 --send --proxy=${PROXY} --chain=1
}

setSpecialRewardAmount() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="setSpecialRewardAmount" --arguments 0x5733502d343863633561 0x2B5E3AF16B1880000 --send --proxy=${PROXY} --chain=1
}

deleteOffersByWallet() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=600000000 --function="deleteOffersByWallet" --arguments 0x3c4007eb0e64506da0e5c9883d94fcd2956d2d7cf9e1de7eb084da3a3cab55d1 --send --proxy=${PROXY} --chain=1
}

setCutPercentage() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="setCutPercentage" --arguments 0x64 --send --proxy=${PROXY} --chain=1
}

setStatusOff() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="setStatus" --arguments 0x00 --send --proxy=${PROXY} --chain=1
}

setAcceptedTokens() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="setAcceptedTokens" --arguments 0x57415445522d396564343030 --send --proxy=${PROXY} --chain=1
}

removeAcceptedTokens() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="removeAcceptedTokens" --arguments 0x4c4b4d45582d3830356538 --send --proxy=${PROXY} --chain=1
}

addBlackListWallet() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="addBlackListWallet" --arguments 0x3c4007eb0e64506da0e5c9883d94fcd2956d2d7cf9e1de7eb084da3a3cab55d1 --send --proxy=${PROXY} --chain=1
}

withdraw() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="withdraw" --arguments 0x05 --send --proxy=${PROXY} --chain=1
}

endAuction() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="endAuction" --arguments 0x05 --send --proxy=${PROXY} --chain=1
}

bidCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

bidBob() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=1000000000000000000 --send --proxy=${PROXY} --chain=1
}

bidEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

bidESDTEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=1
}

bidESDTBob() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785d8a0000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=1
}

bidESDTCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=1
}
buyEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

buyCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

buyESDTEve() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTNFTTransfer" --arguments 0x05 0x45474c442d633365323066 0x01 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

buyESDTCarol() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785D8A0000 0x627579 0x01 0x4b42422d316339353733 0x01 --send --proxy=${PROXY} --chain=1
}

listNFT() {
    erdpy --verbose contract call ${CAROLWALLET} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="ESDTNFTTransfer" \
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
    --send --proxy=${PROXY} --chain=1
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
getRewardBalance() {
    erdpy --verbose contract query ${ADDRESS} --function="getRewardBalance" --proxy=${PROXY}
}