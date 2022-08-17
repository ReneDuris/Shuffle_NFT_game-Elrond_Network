#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();
const NFT: u32 = 1;

#[elrond_wasm::contract]
pub trait Shuffle {
    #[init]
    fn init(&self,token_id: TokenIdentifier, minimum_shuffle_entries:usize) { 

        self.sc_nft_id().set(token_id);
        self.minimum_shuffle_entries().set(minimum_shuffle_entries);
    }

    #[payable("*")]
    #[endpoint(enterShuffle)]
    fn enter_shuffle(&self) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let nonce = payment.token_nonce;
        let token = payment.token_identifier;

        require!(self.nonce_list_user(&caller).is_empty(), "Cannot enter, you can insert only one NFT");
        require!(token == self.sc_nft_id().get(), "Cannot enter with different NFT");
        require!(self.game_begin().get() == false, "Cannot enter, game already shuffled");

        self.nonce_list().insert(nonce);
        self.nonce_list_user(&caller).insert(nonce);
        self.user_list().insert(caller);   
    }

    #[endpoint(leaveShuffle)]
    fn leave_shuffle(&self, token_id: TokenIdentifier,nonce: u64) {
        let caller = self.blockchain().get_caller();

        require!(token_id == self.sc_nft_id().get(), "Cannot withdraw different NFT");
        require!(self.nonce_list_user(&caller).contains(&nonce) == true, "You didnt insert this NFT");
        require!(self.game_begin().get() == false, "Cannot withdraw, game already shuffled");

       self.nonce_list_user(&caller).swap_remove(&nonce);
        self.send().direct_esdt(&caller,&token_id ,nonce, &BigUint::from(NFT));
    }

    #[only_owner]
#[endpoint(beginShuffle)]
fn begin_shuffle(&self) {
    let mut rand_source = RandomnessSource::<Self::Api>::new();
    let index_number = self.user_list().len();
    let minimum_entries = self.minimum_shuffle_entries().get();
    require!(index_number >= minimum_entries, "Required more entries, minimum is not satisfied");
    require!(self.game_begin().get() == false, "Cannot shuffle, already shuffled");

    for user in self.user_list().iter() {
        let rand_nr = rand_source.next_usize_in_range(0usize,index_number);
            self.new_store_index(&user).set(rand_nr);
    }
    self.game_begin().set(true);
}

#[only_owner]
#[endpoint(endGame)]
fn end_game(&self) {
    require!(self.nonce_list().is_empty() == true, "Cannot end game, all entries wasnt claimed");
    self.game_begin().set(false);
    self.user_list().clear();
}

#[endpoint(claimShuffle)]
fn claim_shuffle(&self) {
    let caller = &self.blockchain().get_caller();

    require!(self.user_list().contains(&caller) == true, "You did not entry into Shuffle");
    require!(self.game_begin().get() == true, "Cannot claim, game was not shuffled");

    let index = self.new_store_index(&caller).get();
    let nonce = self.nonce_list().get_by_index(index);
    self.nonce_list().swap_remove(&nonce);
    let token_id = self.sc_nft_id().get();
    self.send().direct_esdt(&caller, &token_id, nonce, &BigUint::from(NFT));
    self.new_store_index(caller).clear();
    self.nonce_list_user(&caller).clear();
    
}

#[view(minimumEntries)]
#[storage_mapper("minimumEntries")]
fn minimum_shuffle_entries(&self) -> SingleValueMapper<usize>;

#[view(nonceList)]
#[storage_mapper("nonceList")]
fn nonce_list(&self) -> UnorderedSetMapper<u64>;

#[view(nonceUserList)]
#[storage_mapper("nonceUserList")]
fn nonce_list_user(&self,user: &ManagedAddress) -> UnorderedSetMapper<u64>;

#[view(userList)]
#[storage_mapper("userList")]
fn user_list(&self) -> UnorderedSetMapper<ManagedAddress>;

#[view(storeIndex)]
#[storage_mapper("storeIndex")]
fn new_store_index(&self,user: &ManagedAddress) -> SingleValueMapper<usize>;

#[view(nftToken)]
#[storage_mapper("nftToken")]
fn sc_nft_id(&self) -> SingleValueMapper<TokenIdentifier>;

#[view(gameBegin)]
#[storage_mapper("gameBegin")]
fn game_begin(&self) -> SingleValueMapper<bool>;
}
