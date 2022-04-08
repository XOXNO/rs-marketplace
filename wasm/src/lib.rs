////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        acceptOffer
        addBlackListWallet
        addRewardBalance
        addWitelistedSC
        bid
        blackListWallets
        buy
        changePrice
        checkOfferSent
        checkTokenOffers
        claimTokens
        defaultRewardAmount
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
        getClaimableTokenNonces
        getClaimableTokens
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
        getOffersCount
        getOnSaleTokensForTicker
        getOriginalOwner
        getPaymentTokenForAuction
        getRewardBalance
        getRewardTicker
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
        setDefaultRewardAmount
        setRewardTicker
        setSpecialRewardAmount
        setStatus
        specialRewardAmount
        whitelistedContracts
        withdraw
        withdrawOffer
    )
}

elrond_wasm_node::wasm_empty_callback! {}
