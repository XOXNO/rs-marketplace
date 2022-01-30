////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        acceptOffer
        addWitelistedSC
        bid
        buy
        checkOfferSent
        checkTokenOffers
        claimTokens
        deleteOffersByWallet
        doesAuctionExist
        doesOfferExist
        endAuction
        getAcceptedTokens
        getAcceptedTokensCount
        getActiveListingsBids
        getAuctionType
        getAuctionedToken
        getBulkListings
        getBulkOffers
        getClaimableAmount
        getCollectionsCount
        getCollectionsListed
        getCurrentWinner
        getCurrentWinningBid
        getDeadline
        getFullAuctionData
        getLastValidAuctionId
        getLastValidOfferId
        getListings
        getListingsByWallet
        getListingsCount
        getMarketplaceCutPercentage
        getMinMaxBid
        getOffers
        getOffersByWallet
        getOnSaleTokensForTicker
        getOriginalOwner
        getPaymentTokenForAuction
        getStartTime
        getStatus
        getTokenAuctionIds
        getTokenItemsForSale
        getTokenItemsForSaleCount
        getTokenItemsQuantityForSale
        getTokenOffersIds
        listing
        offerById
        removeAcceptedTokens
        sendOffer
        setAcceptedTokens
        setCutPercentage
        setStatus
        whitelistedContracts
        withdraw
        withdrawOffer
    )
}

elrond_wasm_node::wasm_empty_callback! {}
