use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{ Deserialize, Serialize };
use std::marker::PhantomData;

use cosmwasm_std::{ Addr, BlockInfo, Coin, CustomMsg, StdResult, Storage };

use cw721::{ ContractInfoResponse, Cw721, Expiration };
use cw_storage_plus::{ Index, IndexList, IndexedMap, Item, Map, MultiIndex };

pub struct Cw721Contract<'a, T, C, E, Q>
    where T: Serialize + DeserializeOwned + Clone, Q: CustomMsg, E: CustomMsg {
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub token_count: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,
    pub withdraw_address: Item<'a, String>,
    pub max_nfts_per_wallet: Item<'a, u64>,
    pub max_supply: Item<'a, u64>,
    pub mint_price_per_nft: Item<'a, Coin>,
    pub wallets_minted_amount: Map<'a, String, u64>,

    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

// This is a signal, the implementations are in other files
impl<'a, T, C, E, Q> Cw721<T, C>
    for Cw721Contract<'a, T, C, E, Q>
    where T: Serialize + DeserializeOwned + Clone, C: CustomMsg, E: CustomMsg, Q: CustomMsg {}

impl<T, C, E, Q> Default
    for Cw721Contract<'static, T, C, E, Q>
    where T: Serialize + DeserializeOwned + Clone, E: CustomMsg, Q: CustomMsg
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
            "withdraw_address",
            "max_nfts_per_wallet",
            "wallets_minted_amount",
            "max_supply",
            "mint_price_per_nft"
        )
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
    where T: Serialize + DeserializeOwned + Clone, E: CustomMsg, Q: CustomMsg
{
    fn new(
        contract_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
        withdraw_address_key: &'a str,
        max_nfts_per_wallet_key: &'a str,
        wallets_minted_amount_key: &'a str,
        max_supply_key: &'a str,
        mint_price_per_nft_key: &'a str
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            token_count: Item::new(token_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            withdraw_address: Item::new(withdraw_address_key),
            max_nfts_per_wallet: Item::new(max_nfts_per_wallet_key),
            max_supply: Item::new(max_supply_key),
            mint_price_per_nft: Item::new(mint_price_per_nft_key),
            wallets_minted_amount: Map::new(wallets_minted_amount_key),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage, sender: &String) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;

        let user_minted = self.wallets_minted_amount.load(storage, sender.clone()).unwrap_or(0);
        self.wallets_minted_amount.save(storage, sender.clone(), &(user_minted + 1))?;

        Ok(val)
    }

    pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? - 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T> where T: Serialize + DeserializeOwned + Clone {
    pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>>
    for TokenIndexes<'a, T>
    where T: Serialize + DeserializeOwned + Clone
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(_pk: &[u8], d: &TokenInfo<T>) -> Addr {
    d.owner.clone()
}
