////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        init
        bid
        buy
        doesAuctionExist
        endAuction
        getAcceptedTokens
        getAcceptedTokensCount
        getActiveListingsBids
        getAuctionType
        getAuctionedToken
        getCurrentWinner
        getCurrentWinningBid
        getDeadline
        getDustAmountLeft
        getFullAuctionData
        getLastValidAuctionId
        getListings
        getListingsByWallet
        getListingsCount
        getLocalBalance
        getMarketplaceCutPercentage
        getMinMaxBid
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
        setStatus
        withdraw
    )
}

elrond_wasm_node::wasm_empty_callback! {}
