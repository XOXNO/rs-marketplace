#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod auction;
use auction::*;

mod events;
pub mod storage;
pub mod views;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT

#[elrond_wasm::contract]
pub trait EsdtNftMarketplace:
    storage::StorageModule + views::ViewsModule + events::EventsModule
{
    #[init]
    fn init(&self, bid_cut_percentage: u64) -> SCResult<()> {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
        self.status().set(&false);
        Ok(())
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(setCutPercentage)]
    fn set_percentage_cut(&self, new_cut_percentage: u64) -> SCResult<()> {
        self.try_set_bid_cut_percentage(new_cut_percentage)
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(setAcceptedTokens)]
    fn set_accepted_tokens(&self, token: TokenIdentifier) -> SCResult<()> {
        self.accepted_tokens().insert(token);
        Ok(())
    }
    #[only_owner]
    #[endpoint(removeAcceptedTokens)]
    fn remove_accepted_tokens(&self, token: TokenIdentifier) -> bool {
        self.accepted_tokens().remove(&token)
    }
    // endpoints - owner-only
    #[only_owner]
    #[endpoint(getDustAmountLeft)]
    fn get_dust_amount_left(&self, token: TokenIdentifier, nonce: u64) {
        let local_balance = self.local_token_balance(token.clone()).get();
        let mut sc_balance = BigUint::zero();
        if (token.is_egld()) {
            sc_balance = self
                .blockchain()
                .get_balance(&self.blockchain().get_sc_address());
        } else if (token.is_valid_esdt_identifier() && token.is_esdt()) {
            sc_balance = self.blockchain().get_esdt_balance(
                &self.blockchain().get_sc_address(),
                &token,
                nonce,
            );
        }
        // send part as cut for contract owner
        let owner = self.blockchain().get_owner_address();
        self.transfer_esdt(
            &owner,
            &token,
            nonce,
            &(sc_balance - local_balance),
            b"Trust Market fees revenue!",
        );
    }

    // endpoints - owner-only
    #[only_owner]
    #[endpoint(setStatus)]
    fn set_status(&self, status: bool) -> SCResult<()> {
        self.status().set(&status);
        Ok(())
    }
    // endpoints

    #[payable("*")]
    #[endpoint(listing)]
    #[allow(clippy::too_many_arguments)]
    fn listing(
        &self,
        #[payment_token] nft_type: TokenIdentifier,
        #[payment_nonce] nft_nonce: u64,
        #[payment_amount] nft_amount: BigUint,
        min_bid: BigUint,
        max_bid: BigUint,
        deadline: u64,
        accepted_payment_token: TokenIdentifier,
        bid: bool,
        #[var_args] opt_sft_max_one_per_payment: OptionalArg<bool>,
        #[var_args] opt_start_time: OptionalArg<u64>,
    ) -> SCResult<u64> {
        require!(self.status().get(), "Global operation enabled!");

        require!(
            self.accepted_tokens().contains(&accepted_payment_token),
            "The payment token is not whitelisted!"
        );

        require!(
            nft_amount >= BigUint::from(NFT_AMOUNT),
            "Must tranfer at least one"
        );

        let current_time = self.blockchain().get_block_timestamp();
        let start_time = match opt_start_time {
            OptionalArg::Some(st) => st,
            OptionalArg::None => current_time,
        };

        let sft_max_one_per_payment = opt_sft_max_one_per_payment
            .into_option()
            .unwrap_or_default();

        if sft_max_one_per_payment || !bid {
            require!(
                min_bid == max_bid,
                "Price must be fixed for this type of auction (min bid equal to max bid)"
            );
        }
        if (!accepted_payment_token.is_egld()) {
            require!(
                accepted_payment_token.is_valid_esdt_identifier(),
                "The payment token is not valid!"
            );
        }

        let opt_max_bid = if max_bid > 0u32 {
            require!(min_bid <= max_bid, "Min bid can't higher than max bid");

            Some(max_bid)
        } else {
            None
        };

        require!(min_bid > 0u32, "Min bid must be higher than 0!");
        require!(
            nft_nonce > 0,
            "Only Semi-Fungible and Non-Fungible tokens can be auctioned"
        );
        require!(
            deadline > current_time || deadline == 0,
            "Deadline can't be in the past"
        );
        if (deadline != 0) {
            require!(
                start_time >= current_time && start_time < deadline,
                "Invalid start time"
            );
        }

        let marketplace_cut_percentage = self.bid_cut_percentage().get();
        let creator_royalties_percentage = self.get_nft_info(&nft_type, nft_nonce).royalties;

        require!(
            &marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );

        let accepted_payment_nft_nonce = 0;

        let auction_id = self.last_valid_auction_id().get() + 1;
        self.last_valid_auction_id().set(&auction_id);

        let auction_type = if nft_amount > BigUint::from(NFT_AMOUNT) {
            match sft_max_one_per_payment {
                true => AuctionType::SftOnePerPayment,
                false => AuctionType::SftAll,
            }
        } else {
            match bid {
                true => AuctionType::NftBid,
                false => AuctionType::Nft,
            }
        };

        if (deadline == 0) {
            require!(
                auction_type == AuctionType::Nft || auction_type == AuctionType::SftOnePerPayment,
                "Deadline is mandatory for this auction type!"
            );
        }
        let auction = Auction {
            auctioned_token: EsdtToken {
                token_type: nft_type.clone(),
                nonce: nft_nonce,
            },
            nr_auctioned_tokens: nft_amount.clone(),
            auction_type,

            payment_token: EsdtToken {
                token_type: accepted_payment_token,
                nonce: accepted_payment_nft_nonce,
            },
            min_bid,
            max_bid: opt_max_bid,
            start_time,
            deadline,

            original_owner: self.blockchain().get_caller(),
            current_bid: BigUint::zero(),
            current_winner: ManagedAddress::zero(),
            marketplace_cut_percentage,
            creator_royalties_percentage,
        };
        // Map ID with Auction Struct
        self.auction_by_id(auction_id).set(&auction);
        self.listings().insert(auction_id); // Push ID to the auctions list
                                            // Add to the owner wallet the new Auction ID
        self.listings_by_wallet(auction.original_owner.clone())
            .insert(auction_id.clone());
        // Insert nonce for sale per collection
        self.token_items_for_sale(nft_type.clone())
            .insert(nft_nonce);
        // Insert auction ID per token and nonce
        self.token_auction_ids(nft_type.clone(), nft_nonce.clone())
            .insert(auction_id);

        self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
            .update(|qt| *qt += &nft_amount.clone());

        //Emit event for new listed token
        self.emit_auction_token_event(auction_id, auction);

        Ok(auction_id)
    }

    #[payable("*")]
    #[endpoint]
    fn bid(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
    ) -> SCResult<()> {
        require!(self.status().get(), "Global operation enabled!");
        let mut auction = self.try_get_auction(auction_id)?;
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            auction.auction_type == AuctionType::SftAll
                || auction.auction_type == AuctionType::NftBid,
            "Cannot bid on this type of auction!"
        );
        require!(
            auction.auctioned_token.token_type == nft_type
                && auction.auctioned_token.nonce == nft_nonce,
            "Auction ID does not match the token!"
        );
        require!(
            auction.original_owner != caller,
            "Cannot bid on your own token!"
        );
        require!(
            current_time >= auction.start_time,
            "Auction hasn't started yet!"
        );
        require!(current_time < auction.deadline, "Auction ended already!");
        require!(
            payment_token == auction.payment_token.token_type,
            "Wrong token used as payment!"
        );
        require!(auction.current_winner != caller, "Can't outbid yourself!");
        require!(
            payment_amount >= auction.min_bid,
            "Bid must be higher than or equal to the min bid!"
        );
        require!(
            payment_amount > auction.current_bid,
            "Bid must be higher than the current winning bid!"
        );

        if let Some(max_bid) = &auction.max_bid {
            require!(
                &payment_amount <= max_bid,
                "Bid must be less than or equal to the max bid!"
            );
        }

        // refund losing bid
        if auction.current_winner != ManagedAddress::zero() {
            self.transfer_esdt(
                &auction.current_winner,
                &auction.payment_token.token_type,
                auction.payment_token.nonce,
                &auction.current_bid,
                b"Trust Market refunded your bid!",
            );
            self.listings_bids(auction.current_winner.clone())
                .remove(&auction_id);
            self.local_token_balance(nft_type.clone())
                .update(|bl| *bl = (bl.clone().sub(auction.current_bid.clone())));
        }

        // update auction bid and winner
        auction.payment_token.nonce = payment_token_nonce;
        auction.current_bid = payment_amount;
        auction.current_winner = caller;
        self.auction_by_id(auction_id).set(&auction);
        self.listings_bids(auction.current_winner.clone())
            .insert(auction_id);
        self.local_token_balance(nft_type.clone())
            .update(|bl| *bl = (bl.clone().add(auction.current_bid.clone())));
        if let Some(max_bid) = &auction.max_bid {
            if (&auction.current_bid == max_bid) {
                self.end_auction(auction_id);
            }
        }

        self.emit_bid_event(auction_id, auction);
        Ok(())
    }

    #[endpoint(endAuction)]
    fn end_auction(&self, auction_id: u64) -> SCResult<()> {
        require!(self.status().get(), "Global operation enabled!");
        let auction = self.try_get_auction(auction_id)?;
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            auction.auction_type == AuctionType::SftAll
                || auction.auction_type == AuctionType::NftBid,
            "Cannot end this type of auction!"
        );
        let deadline_reached = current_time > auction.deadline;
        let max_bid_reached = if let Some(max_bid) = &auction.max_bid {
            &auction.current_bid == max_bid
        } else {
            false
        };

        require!(
            deadline_reached || max_bid_reached,
            "Auction deadline has not passed or the current bid is not equal to the max bid!"
        );

        self.distribute_tokens(&auction, None);
        self.listings_by_wallet(auction.original_owner.clone())
            .remove(&auction_id);
        self.listings_bids(auction.current_winner.clone())
            .remove(&auction_id);
        self.token_auction_ids(
            auction.auctioned_token.token_type.clone(),
            auction.auctioned_token.nonce.clone(),
        )
        .remove(&auction_id);
        self.listings().remove(&auction_id);
        self.auction_by_id(auction_id).clear();
        self.emit_end_auction_event(auction_id, auction);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[payable("*")]
    #[endpoint(buy)]
    fn buy(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        #[var_args] opt_sft_buy_amount: OptionalArg<BigUint>,
    ) -> SCResult<()> {
        require!(self.status().get(), "Global operation enabled!");
        let mut auction = self.try_get_auction(auction_id)?;
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();
        if (payment_token_nonce > 0) {
            let payment_token_info = self.get_nft_info(&payment_token, payment_token_nonce);

            require!(
                payment_token_info.token_type == EsdtTokenType::Fungible
                    || payment_token_info.token_type == EsdtTokenType::Meta,
                "The payment token is invalid!"
            );
        }
        let buy_amount = match opt_sft_buy_amount {
            OptionalArg::Some(amt) => amt,
            OptionalArg::None => BigUint::from(NFT_AMOUNT),
        };
        let total_value = &buy_amount * &auction.min_bid;

        require!(buy_amount > 0, "The amount must be more than 0!");
        require!(
            auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot buy for this type of auction!"
        );
        require!(
            auction.auctioned_token.token_type == nft_type
                && auction.auctioned_token.nonce == nft_nonce,
            "Auction ID does not match the token!"
        );
        require!(
            auction.original_owner != caller,
            "Cannot buy your own token!"
        );
        require!(
            buy_amount <= auction.nr_auctioned_tokens,
            "Not enough quantity available!"
        );
        require!(
            payment_token == auction.payment_token.token_type,
            "Wrong token used as payment"
        );
        require!(
            total_value == payment_amount,
            "Wrong amount paid, must pay equal to the selling price!"
        );
        require!(
            current_time >= auction.start_time,
            "Cannot buy before start time!"
        );
        if (auction.deadline != 0) {
            require!(
                current_time <= auction.deadline,
                "Cannot buy after deadline!"
            );
        }

        auction.current_winner = caller;
        auction.current_bid = payment_amount;
        auction.payment_token.nonce = payment_token_nonce;
        self.distribute_tokens(&auction, Some(&buy_amount));
        auction.nr_auctioned_tokens -= &buy_amount;
        if auction.nr_auctioned_tokens == 0 {
            self.listings_by_wallet(auction.original_owner.clone())
                .remove(&auction_id);
            self.token_auction_ids(nft_type.clone(), nft_nonce.clone())
                .remove(&auction_id);
            self.auction_by_id(auction_id).clear();
            self.listings().remove(&auction_id);
        } else {
            self.auction_by_id(auction_id).set(&auction);
        }

        self.emit_buy_event(auction_id, auction, buy_amount);

        Ok(())
    }

    #[endpoint]
    fn withdraw(&self, auction_id: u64) -> SCResult<()> {
        require!(self.status().get(), "Global operation enabled!");
        let auction = self.try_get_auction(auction_id)?;
        let caller = self.blockchain().get_caller();

        require!(
            auction.original_owner == caller,
            "Only the original owner can withdraw!"
        );
        require!(
            auction.current_winner.is_zero()
                || auction.auction_type == AuctionType::SftOnePerPayment
                || auction.auction_type == AuctionType::Nft,
            "Cannot withdraw, the auction already has bids!"
        );

        self.distribute_tokens(&auction, Option::Some(&auction.nr_auctioned_tokens));

        self.token_auction_ids(
            auction.auctioned_token.token_type.clone(),
            auction.auctioned_token.nonce.clone(),
        )
        .remove(&auction_id);
        self.listings_by_wallet(auction.original_owner.clone())
            .remove(&auction_id);
        self.listings().remove(&auction_id);
        self.auction_by_id(auction_id).clear();
        self.emit_withdraw_event(auction_id, auction);

        Ok(())
    }

    // private

    fn try_get_auction(&self, auction_id: u64) -> SCResult<Auction<Self::Api>> {
        require!(
            self.does_auction_exist(auction_id),
            "Auction does not exist!"
        );
        Ok(self.auction_by_id(auction_id).get())
    }

    fn calculate_cut_amount(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / PERCENTAGE_TOTAL
    }

    fn calculate_winning_bid_split(
        &self,
        auction: &Auction<Self::Api>,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&auction.current_bid, &auction.creator_royalties_percentage);
        let bid_cut_amount =
            self.calculate_cut_amount(&auction.current_bid, &auction.marketplace_cut_percentage);
        let mut seller_amount_to_send = auction.current_bid.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn distribute_tokens(&self, auction: &Auction<Self::Api>, opt_sft_amount: Option<&BigUint>) {
        let nft_type = &auction.auctioned_token.token_type;
        let nft_nonce = auction.auctioned_token.nonce;
        if !auction.current_winner.is_zero() {
            let nft_info = self.get_nft_info(nft_type, nft_nonce);
            let token_id = &auction.payment_token.token_type;
            let nonce = auction.payment_token.nonce;
            let bid_split_amounts = self.calculate_winning_bid_split(auction);

            // send part as cut for contract owner
            let owner = self.blockchain().get_owner_address();
            self.transfer_esdt(
                &owner,
                token_id,
                nonce,
                &bid_split_amounts.marketplace,
                b"Trust Market fees revenue!",
            );
            // send part as royalties to creator
            self.transfer_esdt(
                &nft_info.creator,
                token_id,
                nonce,
                &bid_split_amounts.creator,
                b"Trust Market royalties for your token!",
            );
            // send rest of the bid to original owner
            self.transfer_esdt(
                &auction.original_owner,
                token_id,
                nonce,
                &bid_split_amounts.seller,
                b"Trust Market income!",
            );

            // send NFT to auction winner
            let nft_amount = BigUint::from(NFT_AMOUNT);
            let nft_amount_to_send = match auction.auction_type {
                AuctionType::Nft => &nft_amount,
                AuctionType::NftBid => &nft_amount,
                AuctionType::SftOnePerPayment => match opt_sft_amount {
                    Some(amt) => amt,
                    None => &nft_amount,
                },
                _ => &auction.nr_auctioned_tokens,
            };
            self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .update(|qt| *qt -= nft_amount_to_send.clone());

            if (self
                .token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .get()
                == BigUint::from(0u32))
            {
                self.token_items_for_sale(nft_type.clone())
                    .remove(&nft_nonce);
                self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                    .clear();
            }
            self.transfer_esdt(
                &auction.current_winner,
                nft_type,
                nft_nonce,
                nft_amount_to_send,
                b"Trust Market sent the bought token!",
            );
        } else {
            // return to original owner

            self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .update(|qt| *qt -= &auction.nr_auctioned_tokens);
            let quantity_token = self
                .token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                .get();
            if (quantity_token.eq(&BigUint::from(0u32))) {
                self.token_items_for_sale(nft_type.clone())
                    .remove(&nft_nonce);
                self.token_items_quantity_for_sale(nft_type.clone(), nft_nonce.clone())
                    .clear();
            }
            self.transfer_esdt(
                &auction.original_owner,
                nft_type,
                nft_nonce,
                &auction.nr_auctioned_tokens,
                b"Trust Market returned your token!",
            );
        }
    }

    fn transfer_esdt(
        &self,
        to: &ManagedAddress,
        token_id: &TokenIdentifier,
        nonce: u64,
        amount: &BigUint,
        data: &'static [u8],
    ) {
        self.send().direct(
            to,
            token_id,
            nonce,
            amount,
            self.data_or_empty_if_sc(to, data),
        );
    }

    fn data_or_empty_if_sc(&self, dest: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(dest) {
            &[]
        } else {
            data
        }
    }

    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }

    fn try_set_bid_cut_percentage(&self, new_cut_percentage: u64) -> SCResult<()> {
        require!(
            new_cut_percentage > 0 && new_cut_percentage < PERCENTAGE_TOTAL,
            "Invalid percentage value, should be between 0 and 10,000"
        );

        self.bid_cut_percentage()
            .set(&BigUint::from(new_cut_percentage));

        Ok(())
    }

    // storage
}
