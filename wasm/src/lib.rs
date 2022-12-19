////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        acceptGlobalOffer
        acceptOffer
        addBlackListWallet
        addRewardBalance
        addWitelistedSC
        bid
        blackListWallets
        bulkBuy
        buy
        buyFor
        changePrice
        checkOfferSent
        checkTokenOffers
        claimTokens
        claimTokensForCreator
        cleanExpiredOffers
        declineOffer
        defaultRewardAmount
        deleteOffersByWallet
        doesAuctionExist
        doesOfferExist
        endAuction
        expiredOffersCount
        getAcceptedTokens
        getAcceptedTokensCount
        getActiveListingsBids
        getAuctionType
        getAuctionedToken
        getAuctionedTokenAndOwner
        getAuctionsForTicker
        getBulkListings
        getBulkOffers
        getClaimableAmount
        getClaimableTokenNonces
        getClaimableTokens
        getCollectionGlobalOffers
        getCollectionsCount
        getCollectionsListed
        getCurrentWinner
        getCurrentWinningBid
        getDeadline
        getFullAuctionData
        getGlobalOffer
        getGlobalOffers
        getLastValidAuctionId
        getLastValidGlobalOfferId
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
        getSigner
        getStartTime
        getStatus
        getTokenAuctionIds
        getTokenItemsForSale
        getTokenItemsForSaleCount
        getTokenItemsQuantityForSale
        getTokenOffersIds
        isCollectionListed
        isSCWl
        listing
        offerById
        removeAcceptedTokens
        removeBlackListWallet
        removeWitelistedSC
        returnListing
        sendGlobalOffer
        sendOffer
        setAcceptedTokens
        setCutPercentage
        setDefaultRewardAmount
        setRewardTicker
        setSpecialRewardAmount
        setStatus
        specialRewardAmount
        userCollectionGlobalOffers
        userGlobalOffers
        whitelistedContracts
        withdraw
        withdrawGlobalOffer
        withdrawGlobalOffers
        withdrawOffer
    )
}

elrond_wasm_node::wasm_empty_callback! {}
