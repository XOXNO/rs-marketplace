// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           93
// Async Callback (empty):               1
// Total number of exported functions:  95

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        listing
        bid
        endAuction
        buy
        buyFor
        bulkBuy
        withdraw
        changePrice
        getSigner
        getMarketplaceCutPercentage
        getOffersByWallet
        checkOfferSent
        getListingsByWallet
        getActiveListingsBids
        getTokenItemsForSale
        getTokenAuctionIds
        getTokenOffersIds
        getTokenItemsQuantityForSale
        getAcceptedTokens
        blackListWallets
        whitelistedContracts
        getClaimableAmount
        getClaimableTokens
        getClaimableTokenNonces
        getCollectionsListed
        getOffers
        getStatus
        offerById
        getLastValidAuctionId
        getRewardBalance
        getRewardTicker
        specialRewardAmount
        defaultRewardAmount
        getLastValidOfferId
        getLastValidGlobalOfferId
        getGlobalOffers
        getGlobalOffer
        getCollectionGlobalOffers
        userGlobalOffers
        userCollectionGlobalOffers
        getListingsCount
        getOffersCount
        getListings
        getCollectionsCount
        isCollectionListed
        expiredOffersCount
        getAcceptedTokensCount
        getTokenItemsForSaleCount
        getOnSaleTokensForTicker
        getAuctionsForTicker
        checkTokenOffers
        getBulkOffers
        getBulkListings
        doesAuctionExist
        doesGlobalOfferExist
        doesOfferExist
        isSCWl
        getAuctionedToken
        getAuctionedTokenAndOwner
        getAuctionType
        getPaymentTokenForAuction
        getMinMaxBid
        getStartTime
        getDeadline
        getOriginalOwner
        getCurrentWinningBid
        getCurrentWinner
        getFullAuctionData
        acceptOffer
        declineOffer
        withdrawOffer
        sendOffer
        sendGlobalOffer
        withdrawGlobalOffer
        acceptGlobalOffer
        returnListing
        withdrawGlobalOffers
        deleteOffersByWallet
        cleanExpiredOffers
        addRewardBalance
        setRewardTicker
        setSpecialRewardAmount
        setDefaultRewardAmount
        setAcceptedTokens
        removeAcceptedTokens
        addWitelistedSC
        removeWitelistedSC
        setStatus
        setCutPercentage
        claimTokensForCreator
        addBlackListWallet
        removeBlackListWallet
        claimTokens
    )
}

elrond_wasm_node::wasm_empty_callback! {}
