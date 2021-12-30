////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        init
        addWitelistedSC
        bid
        buy
        claimTokens
        doesAuctionExist
        endAuction
        getAcceptedTokens
        getAcceptedTokensCount
        getActiveListingsBids
        getAuctionType
        getAuctionedToken
        getClaimableAmount
        getCollectionKeybase
        getCollectionsCount
        getCollectionsListed
        getCurrentWinner
        getCurrentWinningBid
        getDeadline
        getFullAuctionData
        getLastValidAuctionId
        getListings
        getListingsByWallet
        getListingsCount
        getLocalBalance
        getMarketplaceCutPercentage
        getMinMaxBid
        getOnSaleTokensForTicker
        getOriginalOwner
        getPaymentTokenForAuction
        getStartTime
        getStatus
        getTokenAuctionIds
        getTokenBalanceDifference
        getTokenItemsForSale
        getTokenItemsForSaleCount
        getTokenItemsQuantityForSale
        listing
        removeAcceptedTokens
        setAcceptedTokens
        setCutPercentage
        setKeybase
        setStatus
        whitelistedContracts
        withdraw
    )
}

elrond_wasm_node::wasm_empty_callback! {}
