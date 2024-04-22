
   
ALICE="/Users/truststaking/Desktop/PEM/marketplace.pem"
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
DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-mainnet)
PROXY=https://xoxnogateway.livelysmoke-45e2a824.westeurope.azurecontainerapps.io

SHARD2WrappingWEGLD=erd1qqqqqqqqqqqqqpgqmuk0q2saj0mgutxm4teywre6dl8wqf58xamqdrukln

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${OWNER} --gas-limit=125000000 --arguments 0x64 --send --outfile="deploy-mainnet.interaction.json" --proxy=${PROXY} --chain=1 || return

    TRANSACTION=$(mxpy data parse --file="deploy-mainnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="deploy-mainnet.interaction.json" --expression="data['emitted_tx']['address']")

    mxpy data store --key=address-mainnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-mainnet --value=${TRANSACTION}

    echo ""
}

upgrade() {
    echo "Smart contract address: ${ADDRESS}"
    mxpy --verbose contract upgrade ${ADDRESS} --bytecode="/Users/mihaieremia/GitHub/marketplace/output/xoxno-protocol.wasm" --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=250000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=1 || return
}

setExtraFees() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=40000000 --function="setExtraFees" --arguments "str:CLDBRKRS-e1ae14" 0x96 erd1f4fvzkka27xpc3ec4mzf8t939zw5d0amwyk9fm4dqdve55vht26qa8hnmu --send --proxy=${PROXY} --chain=1
}

setRoyaltiesReverted() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="setRoyaltiesReverted" --arguments "str:COW-cd463d" 0x00 --send --proxy=${PROXY} --chain=1
}


addWitelistedSC() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=40000000 --function="addWitelistedSC" --arguments erd1qqqqqqqqqqqqqpgqte3ntwhq8xcspmqf0q5sveh5rhv3ng8pu76ss8mv96 --send --proxy=${PROXY} --chain=1
}

claimLeftOverDust() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=40000000 --function="claimLeftOverDust" --arguments str:MEX-455c57 43693206751350000000000 --send --proxy=${PROXY} --chain=1
}


unLockTokens() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=600000000 --function="unLockTokens" --arguments erd1qqqqqqqqqqqqqpgqjpt0qqgsrdhp2xqygpjtfrpwf76f9nvg2jpsg4q7th str:LKMEX-aab910 0x1e1f9c 0x1edc09 0x4507cd 0x461fa6 0x22e4ef 0x35d1ed 0x3c430e 0x2c40f8 0x3d7c0f 0x246f45 0x445b36 --send --proxy=${PROXY} --chain=1
}

removeWitelistedSC() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=15000000 --function="removeWitelistedSC" --arguments erd1qqqqqqqqqqqqqpgq75w3kkazfdvfxrl3mvwd3cp7yhzucnuv92rsjq304j --send --proxy=${PROXY} --chain=1
}
setStatusOn() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="setStatus" --arguments 0x01 --send --proxy=${PROXY} --chain=1
}

setStatusOff() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="setStatus" --arguments 0x00 --send --proxy=${PROXY} --chain=1
}

setRewardTicker() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="setRewardTicker" --arguments 0x424f4245522d396562373634 --send --proxy=${PROXY} --chain=1
}

setDefaultRewardAmount() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="setDefaultRewardAmount" --arguments 0x03bd913e6c1df40000 --send --proxy=${PROXY} --chain=1
}

setSpecialRewardAmount() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="setSpecialRewardAmount" --arguments 0x475541524449414e2d336436363335 0x2B5E3AF16B1880000 --send --proxy=${PROXY} --chain=1
}

setCutPercentage() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=6500000 --function="setCutPercentage" --arguments 0x64 --send --proxy=${PROXY} --chain=1
}

setAcceptedTokens() {
    mxpy contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="setAcceptedTokens" --arguments str:SEGLD-3ad2d0 --send --proxy=${PROXY} --chain=1
}

unFreezeAuctionId() {
    mxpy contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="unFreezeAuctionId" --arguments 0x0ffb2f --send --proxy=${PROXY} --chain=1
}

unFreezeAllAuctionIds() {
    mxpy contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=50000000 --function="unFreezeAllAuctionIds" --send --proxy=${PROXY} --chain=1
}

enableRoyaltiesReverted() {
    mxpy contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="enableRoyaltiesReverted" --arguments str:COW-cd463d --send --proxy=${PROXY} --chain=1
}

removeRoyaltiesReverted() {
    mxpy contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="removeRoyaltiesReverted" --arguments str:COW-cd463d --send --proxy=${PROXY} --chain=1
}

removeAcceptedTokens() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 --function="removeAcceptedTokens" --arguments str:MEX-455c57 --send --proxy=${PROXY} --chain=1
}

deleteOffersByWallet() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=600000000 --function="deleteOffersByWallet" --arguments erd1vkk9dc4tu4j8wacykqy3fduereaa0qlxutk08ueklhvf6pvgjgasvfwc7h --send --proxy=${PROXY} --chain=1
}

addBlackListWallet() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce  --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="addBlackListWallet" --arguments erd15kgvfgxepluq4y95dcfrl9azqsjr3dhpmmz85anj0a9lxhrz569skmjmkx --send --proxy=${PROXY} --chain=1
}

removeBlackListWallet() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --gas-limit=10000000 --function="removeBlackListWallet" --arguments erd15kgvfgxepluq4y95dcfrl9azqsjr3dhpmmz85anj0a9lxhrz569skmjmkx --send --proxy=${PROXY} --chain=1
}

withdraw() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="withdraw" --arguments 0x05 --send --proxy=${PROXY} --chain=1
}

endAuction() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="endAuction" --arguments 0x05 --send --proxy=${PROXY} --chain=1
}

bidCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

bidBob() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=1000000000000000000 --send --proxy=${PROXY} --chain=1
}

bidEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="bid" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

bidESDTEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=1
}

bidESDTBob() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785d8a0000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=1
}

bidESDTCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x00D529AE9E860000 0x626964 0x05 0x4b42422d316339353733 0x04 --send --proxy=${PROXY} --chain=1
}
buyEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

buyCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="buy" --arguments 0x05 0x4b42422d316339353733 0x05 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

buyESDTEve() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTNFTTransfer" --arguments 0x05 0x45474c442d633365323066 0x01 --value=100000000000000000 --send --proxy=${PROXY} --chain=1
}

buyESDTCarol() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${EVE} --gas-limit=35000000 --function="ESDTTransfer" --arguments 0x45474c442d633365323066 0x016345785D8A0000 0x627579 0x01 0x4b42422d316339353733 0x01 --send --proxy=${PROXY} --chain=1
}

listNFT() {
    mxpy --verbose contract call ${CAROLWALLET} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="ESDTNFTTransfer" \
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
    mxpy --verbose contract query ${ADDRESS} --function="getListingsCount" --proxy=${PROXY}
}

getOffersCount() {
    mxpy --verbose contract query ${ADDRESS} --function="getOffersCount" --proxy=${PROXY}
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
    mxpy --verbose contract query ${ADDRESS} --function="getTokenItemsForSale" --arguments str:MICE-a0c447 --proxy=${PROXY}
}

getTokenItemsQuantityForSale() {
    mxpy --verbose contract query ${ADDRESS} --function="getTokenItemsQuantityForSale" --arguments 0x4b42422d316339353733 0x05 --proxy=${PROXY}
}

getTokenItemsForSaleCount() {
    mxpy --verbose contract query ${ADDRESS} --function="getTokenItemsForSaleCount" --arguments str:MICE-a0c447 --proxy=${PROXY}
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

getRewardBalance() {
    mxpy --verbose contract query ${ADDRESS} --function="getRewardBalance" --proxy=${PROXY}
}

specialRewardAmount() {
    mxpy --verbose contract query ${ADDRESS} --function="specialRewardAmount" --proxy=${PROXY}
}

defaultRewardAmount() {
    mxpy --verbose contract query ${ADDRESS} --function="defaultRewardAmount" --proxy=${PROXY}
}

getRewardTicker() {
    mxpy --verbose contract query ${ADDRESS} --function="getRewardTicker" --proxy=${PROXY}
}

offerById() {
    mxpy --verbose contract query ${ADDRESS} --function="offerById" --arguments 0x0114e5 --proxy=${PROXY}
}

getLastValidGlobalOfferId() {
    mxpy --verbose contract query ${ADDRESS} --function="getLastValidGlobalOfferId" --proxy=${PROXY}
}
getGlobalOffers() {
    mxpy --verbose contract query ${ADDRESS} --function="getGlobalOffers" --proxy=${PROXY}
}

getOffersByWallet() {
    mxpy --verbose contract query ${ADDRESS} --function="getOffersByWallet" --arguments erd1wrr0gvvgpevwg2plphsqzw022wppf726ja94fw8dkrv84vslaarsdqpyt4 --proxy=${PROXY}
}

returnListing() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=35000000 --function="returnListing" --arguments 538990 --send --proxy=${PROXY} --chain=1
}

getGlobalOffer() {
    mxpy --verbose contract query ${ADDRESS} --function="getGlobalOffer" --arguments 5612 --proxy=${PROXY}
}
