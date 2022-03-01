use elrond_wasm::contract_base::{ContractBase};
use elrond_wasm::elrond_codec::multi_types::OptionalValue;
use elrond_wasm::types::{
    BigUint,
};

use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::*,
};
use esdt_nft_marketplace::auction::AuctionType;
use esdt_nft_marketplace::storage::StorageModule;
use esdt_nft_marketplace::views::ViewsModule;
use esdt_nft_marketplace::*;

const SC_WASM_PATH: &'static str = "output/esdt-nft-marketplace.wasm";
fn init() -> BlockchainStateWrapper {
    let wrapper = BlockchainStateWrapper::new();
    return wrapper;
}

#[test]
fn list_nft_bid_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();
    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(1u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn list_nft_sale_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();
    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(1u32),
            BigUint::from(1u32),
            deadline,
            managed_token_id!(&b""[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type.eq(&AuctionType::Nft), true);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn list_sft_bid_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(2);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(2u32),
            BigUint::from(10u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::SftAll);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(2));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn list_sft_bid_as_nft_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(2);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(10u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::Nft);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn list_sft_all_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();
    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(20);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(2u32),
            BigUint::from(1u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::SftAll);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(2));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn list_sft_one_per_payment_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();
    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(20);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(5u32),
            BigUint::from(10u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::Some(true),
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::SftOnePerPayment);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(5));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn list_sft_one_per_payment_as_nft_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(10u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::Some(true),
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::NftBid);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
}

#[test]
fn buy_list_nft_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let payment = &rust_biguint!(100);
    let cut_fee = 1000;
    let creator_nft = wrapper.create_user_account(&rust_biguint!(0));
    let owner_sc = wrapper.create_user_account(&rust_biguint!(0));
    let seller = wrapper.create_user_account(&rust_biguint!(0));
    let buyer = wrapper.create_user_account(&rust_biguint!(1000));
    let scc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &scc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&scc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(1000));
    });

    wrapper.execute_tx(&owner_sc, &scc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b""[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&scc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc.accepted_tokens().contains(&managed_token_id!(&b""[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    
    wrapper.set_nft_balance_all_properties(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
        1000u64,
        Option::Some(&creator_nft),
        Option::None,
        Option::None,
        &([Vec::<u8>::new()]),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &scc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(100u32),
            BigUint::from(100u32),
            deadline,
            managed_token_id!(&b""[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&scc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.original_owner, managed_address!(&seller));
        assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
        assert_eq!(auction.auction_type.eq(&AuctionType::Nft), true);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
        assert_eq!(
            sc.blockchain().get_owner_address(),
            managed_address!(&owner_sc)
        );
        assert_eq!(
            &sc.blockchain().get_sc_address().to_address(),
            scc.address_ref()
        );
    });

    wrapper.check_nft_balance(
        scc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.execute_tx(&buyer, &scc, &payment, |sc| {
        let _ = sc.buy(
            managed_token_id!(&b""[..]),
            0,
            managed_biguint!(100u64),
            1,
            managed_token_id!(token_id),
            nft_nonce,
            OptionalValue::None,
        );
    })
    .assert_ok();

    wrapper.check_nft_balance(
        scc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&buyer, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_egld_balance(&seller, &rust_biguint!(80));
    wrapper.check_egld_balance(&buyer, &rust_biguint!(900));
    wrapper.check_egld_balance(&creator_nft, &rust_biguint!(10));
    wrapper.check_egld_balance(&owner_sc, &rust_biguint!(10));
    let _ = wrapper.execute_query(&scc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 0);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), false);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), false);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), false);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(0));
    });
}

#[test]
fn buy_list_nft_esdt_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let payment = &rust_biguint!(100);
    let cut_fee = 1000;
    let creator_nft = wrapper.create_user_account(&rust_biguint!(0));
    let owner_sc = wrapper.create_user_account(&rust_biguint!(0));
    let seller = wrapper.create_user_account(&rust_biguint!(0));
    let buyer = wrapper.create_user_account(&rust_biguint!(0));
    wrapper.set_esdt_balance(&buyer, &b"ESDT-123456"[..], &rust_biguint!(1000));
    let scc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &scc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&scc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(1000));
    });

    wrapper.execute_tx(&owner_sc, &scc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"ESDT-123456"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&scc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"ESDT-123456"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let payment_token = &b"ESDT-123456"[..];
    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    
    wrapper.set_nft_balance_all_properties(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
        1000u64,
        Option::Some(&creator_nft),
        Option::None,
        Option::None,
        &([Vec::<u8>::new()]),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &scc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(100u32),
            BigUint::from(100u32),
            deadline,
            managed_token_id!(&b"ESDT-123456"[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&scc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.original_owner, managed_address!(&seller));
        assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
        assert_eq!(auction.auction_type.eq(&AuctionType::Nft), true);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
        assert_eq!(
            sc.blockchain().get_owner_address(),
            managed_address!(&owner_sc)
        );
        assert_eq!(
            &sc.blockchain().get_sc_address().to_address(),
            scc.address_ref()
        );
    });

    wrapper.check_nft_balance(
        scc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.execute_esdt_transfer(&buyer, &scc, &payment_token, 0, payment, |sc| {
        let _ = sc.buy(
            managed_token_id!(payment_token),
            0,
            managed_biguint!(100u64),
            1,
            managed_token_id!(token_id),
            nft_nonce,
            OptionalValue::None,
        );  
    })
    .assert_ok();
    wrapper.check_nft_balance(
        scc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&buyer, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_esdt_balance(&seller, payment_token, &rust_biguint!(80));
    wrapper.check_esdt_balance(&buyer, payment_token, &rust_biguint!(900));
    wrapper.check_esdt_balance(&creator_nft, payment_token, &rust_biguint!(10));
    wrapper.check_esdt_balance(&owner_sc, payment_token, &rust_biguint!(10));
    let _ = wrapper.execute_query(&scc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 0);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), false);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), false);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), false);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(0));
    });
}

#[test]
fn buy_list_nft_esdt_meta_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let payment = &rust_biguint!(100);
    let cut_fee = 1000;
    let creator_nft = wrapper.create_user_account(&rust_biguint!(0));
    let owner_sc = wrapper.create_user_account(&rust_biguint!(0));
    let seller = wrapper.create_user_account(&rust_biguint!(0));
    let buyer = wrapper.create_user_account(&rust_biguint!(0));
    wrapper.set_nft_balance(&buyer, &b"ESDT-123456"[..], 5u64, &rust_biguint!(1000.0000), &"");
    let scc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &scc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&scc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(1000));
    });

    wrapper.execute_tx(&owner_sc, &scc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"ESDT-123456"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&scc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"ESDT-123456"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let payment_token = &b"ESDT-123456"[..];
    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    
    wrapper.set_nft_balance_all_properties(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
        1000u64,
        Option::Some(&creator_nft),
        Option::None,
        Option::None,
        &([Vec::<u8>::new()]),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &scc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(100u32),
            BigUint::from(100u32),
            deadline,
            managed_token_id!(&b"ESDT-123456"[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&scc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.original_owner, managed_address!(&seller));
        assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
        assert_eq!(auction.auction_type.eq(&AuctionType::Nft), true);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
        assert_eq!(
            sc.blockchain().get_owner_address(),
            managed_address!(&owner_sc)
        );
        assert_eq!(
            &sc.blockchain().get_sc_address().to_address(),
            scc.address_ref()
        );
    });

    wrapper.check_nft_balance(
        scc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.execute_esdt_transfer(&buyer, &scc, &payment_token, 5u64, payment, |sc| {
        let _ = sc.buy(
            managed_token_id!(payment_token),
            5u64,
            managed_biguint!(100u64),
            1,
            managed_token_id!(token_id),
            nft_nonce,
            OptionalValue::None,
        );
    })
    .assert_ok();

    wrapper.check_nft_balance(
        scc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&buyer, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(
        &seller,
        payment_token,
        5,
        &rust_biguint!(80),
        &(),
    );
    wrapper.check_nft_balance(
        &buyer,
        payment_token,
        5,
        &rust_biguint!(900),
        &(),
    );
    wrapper.check_nft_balance(
        &creator_nft,
        payment_token,
        5,
        &rust_biguint!(10),
        &(),
    );
    wrapper.check_nft_balance(
        &owner_sc,
        payment_token,
        5,
        &rust_biguint!(10),
        &(),
    );
}

#[test]
fn withdraw_list_nft_sale_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);     
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(1u32),
            BigUint::from(1u32),
            deadline,
            managed_token_id!(&b""[..]),
            false,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type.eq(&AuctionType::Nft), true);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.execute_tx(&seller, &sc, &rust_zero, |sc| {
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.withdraw(last_auctiton_id);
        
    })
    .assert_ok();

    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
}

#[test]
fn withdraw_list_sft_all_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(2));
    let seller = wrapper.create_user_account(&rust_biguint!(1));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(18);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(18u32),
            BigUint::from(1u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::SftAll);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(18));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &rust_biguint!(0),
        &(),
    );
    wrapper.execute_tx(&seller, &sc, &rust_zero, |sc| {
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.withdraw(last_auctiton_id);
        
    })
    .assert_ok();

    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
}

#[test]
fn end_bid_sft_bid_pass() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 2500;
    let owner_sc = wrapper.create_user_account(&rust_biguint!(0));
    let seller = wrapper.create_user_account(&rust_biguint!(0));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(2500));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b""[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc.accepted_tokens().contains(&managed_token_id!(&b""[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(123456789);
    let nft_balance = rust_biguint!(2);
    let nft_balance_empty = rust_biguint!(0);
    

    wrapper.set_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(2u32),
            BigUint::from(1u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::None,
            OptionalValue::None,
        );
        
        // assert_eq!(res.err().unwrap(), StaticSCError::from("The payment token is not valid!"));
        
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type, AuctionType::SftAll);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(2));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );

    wrapper.set_block_timestamp(1234569890);
    wrapper.execute_tx(&seller, &sc, &rust_zero, |sc| {
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.end_auction(last_auctiton_id); 
    })
    .assert_ok();

    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
}

#[test]
fn bid_for_nft_max_amount_from2bidders() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 1000;
    let creator_nft = wrapper.create_user_account(&rust_biguint!(0));
    let owner_sc = wrapper.create_user_account(&rust_biguint!(0));
    let seller = wrapper.create_user_account(&rust_biguint!(0));
    let bidder1 = wrapper.create_user_account(&rust_biguint!(1000));
    let bidder2 = wrapper.create_user_account(&rust_biguint!(1000));
    // let bidder1 = wrapper.create_user_account(&rust_biguint!(1000));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Init deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(1000));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true);
        
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    
    wrapper.set_nft_balance_all_properties(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
        1000u64,
        Option::Some(&creator_nft),
        Option::None,
        Option::None,
        &([Vec::<u8>::new()]),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(1u32),
            BigUint::from(10u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::None,
            OptionalValue::None,
        );
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.execute_tx(&bidder1, &sc, &rust_biguint!(2), |sc| {
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.bid(
            managed_token_id!(&b""[..]),
            0,
            managed_biguint!(2u64),
            last_auctiton_id,
            managed_token_id!(token_id),
            nft_nonce,
        );
        
        
    })
    .assert_ok();

    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.original_owner, managed_address!(&seller));
        assert_eq!(auction.current_winner, managed_address!(&bidder1));
        assert_eq!(auction.current_bid, managed_biguint!(2u64));
        assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
        assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });

    wrapper.check_egld_balance(&bidder1, &rust_biguint!(998));
    wrapper.execute_tx(&bidder2, &sc, &rust_biguint!(10), |sc| {
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.bid(
            managed_token_id!(&b""[..]),
            0,
            managed_biguint!(10u64),
            last_auctiton_id,
            managed_token_id!(token_id),
            nft_nonce,
        ); 
    })
    .assert_ok();

    wrapper.check_egld_balance(&bidder1, &rust_biguint!(1000));
    // let _ = wrapper.execute_query(&sc, |sc| {
    //     let listings_count = sc.get_listings_count();
    //     assert_eq!(listings_count, 1);
    //     let last_auctiton_id = sc.last_valid_auction_id().get();
    //     assert_eq!(last_auctiton_id, 1);
    //     let auction = sc.auction_by_id(last_auctiton_id).get();
    //     assert_eq!(auction.original_owner, managed_address!(&seller));
    //     assert_eq!(auction.current_winner, managed_address!(&bidder2));
    //     assert_eq!(auction.current_bid, managed_biguint!(10u64));
    //     assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
    //     assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
    //     let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
    //     assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
    //     let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
    //     assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
    //     let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
    //     assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
    //     let token_items_quantity_for_sale = sc
    //         .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
    //         .get();
    //     assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    // });
    wrapper.check_egld_balance(&bidder2, &rust_biguint!(990));

    // wrapper.set_block_timestamp(1236567890);
    // wrapper.execute_tx(&seller, &sc, &rust_zero, |sc| {
    //     let last_auctiton_id = sc.last_valid_auction_id().get();
    //     let res = sc.end_auction(last_auctiton_id);
    //     
    //     // assert_sc_error!(res, b"Auction deadline has not passed or the current bid is not equal to the max bid!");
    //     
    // });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&bidder2, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_egld_balance(&bidder2, &rust_biguint!(990));
    wrapper.check_egld_balance(&seller, &rust_biguint!(8));
    wrapper.check_egld_balance(&creator_nft, &rust_biguint!(1));
    wrapper.check_egld_balance(&owner_sc, &rust_biguint!(1));
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 0);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), false);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), false);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), false);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(0));
    });
}

#[test]
fn bid_for_nft_with_end_auction_after_deadline() {
    let mut wrapper = init();
    let rust_zero = &rust_biguint!(0);
    let cut_fee = 1000;
    let creator_nft = wrapper.create_user_account(&rust_biguint!(0));
    let owner_sc = wrapper.create_user_account(&rust_biguint!(0));
    let seller = wrapper.create_user_account(&rust_biguint!(0));
    let bidder1 = wrapper.create_user_account(&rust_biguint!(1000));
    let bidder2 = wrapper.create_user_account(&rust_biguint!(1000));
    let sc = wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_sc),
        esdt_nft_marketplace::contract_obj,
        SC_WASM_PATH,
    );

    // Initt deploy
    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.init(cut_fee);    
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let cut_fee = sc.bid_cut_percentage().get();
        let is_active = sc.status().get();
        assert_eq!(accepted_tokens, 0);
        assert_eq!(is_active, false);
        assert_eq!(cut_fee, managed_biguint!(1000));
    });

    wrapper.execute_tx(&owner_sc, &sc, &rust_zero, |sc| {
        let _ = sc.set_accepted_tokens(managed_token_id!(&b"EGLD"[..]));
        let _ = sc.status().set(&true); 
    })
    .assert_ok();

    // Check initial state after deploy
    let _ = wrapper.execute_query(&sc, |sc| {
        let accepted_tokens = sc.get_accepted_tokens_count();
        let status = sc.status().get();
        let accepted_token = sc
            .accepted_tokens()
            .contains(&managed_token_id!(&b"EGLD"[..]));
        assert_eq!(accepted_tokens, 1);
        assert_eq!(accepted_token, true);
        assert_eq!(status, true);
    });

    let token_id = &b"NFT-123456"[..];
    let nft_nonce = 1;
    let deadline = 1234567890;
    wrapper.set_block_timestamp(100);
    let nft_balance = rust_biguint!(1);
    let nft_balance_empty = rust_biguint!(0);
    
    wrapper.set_nft_balance_all_properties(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
        1000u64,
        Option::Some(&creator_nft),
        Option::None,
        Option::None,
        &([Vec::<u8>::new()]),
    );
    wrapper.check_nft_balance(&seller, token_id, nft_nonce, &nft_balance, &());
    wrapper.execute_esdt_transfer(&seller, &sc, token_id, nft_nonce, &nft_balance, |sc| {
        let _ = sc.listing(
            managed_token_id!(token_id),
            nft_nonce,
            BigUint::from(1u32),
            BigUint::from(1u32),
            BigUint::from(11u32),
            deadline,
            managed_token_id!(&b""[..]),
            true,
            OptionalValue::None,
            OptionalValue::None,
        );
    })
    .assert_ok();

    // // Check after the listing
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
        assert_eq!(auction.original_owner, managed_address!(&seller));
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance,
        &(),
    );
    wrapper.check_nft_balance(
        &seller,
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.execute_tx(&bidder1, &sc, &rust_biguint!(2), |sc| {
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.bid(
            managed_token_id!(&b""[..]),
            0,
            managed_biguint!(2u64),
            last_auctiton_id,
            managed_token_id!(token_id),
            nft_nonce,
        );
    })
    .assert_ok();

    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.original_owner, managed_address!(&seller));
        assert_eq!(auction.current_winner, managed_address!(&bidder1));
        assert_eq!(auction.current_bid, managed_biguint!(2u64));
        assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
        assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });

    wrapper.check_egld_balance(&bidder1, &rust_biguint!(998));
    wrapper.execute_tx(&bidder2, &sc, &rust_biguint!(10), |sc| {
        
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.bid(
            managed_token_id!(&b""[..]),
            0,
            managed_biguint!(10u64),
            last_auctiton_id,
            managed_token_id!(token_id),
            nft_nonce,
        );
    })
    .assert_ok();

    wrapper.check_egld_balance(&bidder1, &rust_biguint!(1000));
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 1);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let auction = sc.auction_by_id(last_auctiton_id).get();
        assert_eq!(auction.original_owner, managed_address!(&seller));
        assert_eq!(auction.current_winner, managed_address!(&bidder2));
        assert_eq!(auction.current_bid, managed_biguint!(10u64));
        assert_eq!(auction.creator_royalties_percentage, managed_biguint!(1000));
        assert_eq!(auction.auction_type.eq(&AuctionType::NftBid), true);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), true);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), true);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), true);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(1));
    });
    wrapper.check_egld_balance(&bidder2, &rust_biguint!(990));

    wrapper.set_block_timestamp(1236567890);
    wrapper.execute_tx(&seller, &sc, &rust_zero, |sc| {
        let last_auctiton_id = sc.last_valid_auction_id().get();
        let _ = sc.end_auction(last_auctiton_id); 
    })
    .assert_ok();

    wrapper.check_nft_balance(
        sc.address_ref(),
        token_id,
        nft_nonce,
        &nft_balance_empty,
        &(),
    );
    wrapper.check_nft_balance(&bidder2, token_id, nft_nonce, &nft_balance, &());
    wrapper.check_egld_balance(&bidder2, &rust_biguint!(990));
    wrapper.check_egld_balance(&seller, &rust_biguint!(8));
    wrapper.check_egld_balance(&creator_nft, &rust_biguint!(1));
    wrapper.check_egld_balance(&owner_sc, &rust_biguint!(1));
    let _ = wrapper.execute_query(&sc, |sc| {
        let listings_count = sc.get_listings_count();
        assert_eq!(listings_count, 0);
        let last_auctiton_id = sc.last_valid_auction_id().get();
        assert_eq!(last_auctiton_id, 1);
        let listings_by_wallet = sc.listings_by_wallet(managed_address!(&seller));
        assert_eq!(listings_by_wallet.contains(&last_auctiton_id), false);
        let token_items_for_sale = sc.token_items_for_sale(managed_token_id!(token_id));
        assert_eq!(token_items_for_sale.contains(&nft_nonce), false);
        let token_auction_ids = sc.token_auction_ids(managed_token_id!(token_id), nft_nonce);
        assert_eq!(token_auction_ids.contains(&last_auctiton_id), false);
        let token_items_quantity_for_sale = sc
            .token_items_quantity_for_sale(managed_token_id!(token_id), nft_nonce)
            .get();
        assert_eq!(token_items_quantity_for_sale, managed_biguint!(0));
    });
}
