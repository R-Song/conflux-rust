//////////////////////////////////////////////////////////////////////
/* Signal and Slots begin */

// High level overview:
// This source file provides the method for storing information in the state with respect to
// signals and slots. SignalLocation and SlotLocation define the locations in the state trie.
// SignalInfo and SlotInfo are held in the state information of the account that owns them.
// Slot is the structure appended to a signal. It's purpose is to aid in the generation of
// slot transactions, which are described by SlotTx.

use crate::{bytes::Bytes};
use cfx_types::{Address, U256};
use serde::{Deserialize, Serialize};

// SignalLocation and SlotLocation.
// Structs that keeps track of the location of a signal or slot on the network.
// The two types are the same. We keep them seperate just for readability.
#[derive(
    Clone, Debug, RlpDecodable, RlpEncodable, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct SignalLocation {
    address: Address,
    signal_key: Bytes,
}

impl SignalLocation {
    pub fn new(owner: &Address, signal_key: &[u8]) -> Self {
        let new = SignalLocation {
            address: owner.clone(),
            signal_key: Bytes::from(signal_key),
        };
        new
    }
    // Getters
    pub fn address(&self) -> &Address {
        &self.address
    }
    pub fn signal_key(&self) -> &Bytes {
        &self.signal_key
    }
}

#[derive(
    Clone, Debug, RlpDecodable, RlpEncodable, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct SlotLocation {
    address: Address,
    contract_address: Address,
    slot_key: Bytes,
}

impl SlotLocation {
    pub fn new(owner: &Address, contract: &Address, slot_key: &[u8]) -> Self {
        let new = SlotLocation {
            address: owner.clone(),
            contract_address: contract.clone(),
            slot_key: Bytes::from(slot_key),
        };
        new
    }
    // Getters
    pub fn address(&self) -> &Address {
        &self.address
    }
    pub fn contract_address(&self) -> &Address {
        &self.contract_address
    }
    pub fn slot_key(&self) -> &Bytes {
        &self.slot_key
    }
}

// SignalInfo. Holds the mapping of a signal to a list of slots that are subscribed to it. This info
// is used when a signal is emitted. The list of slots is modified accodingly when a slot binds to it.
#[derive(
    Clone, Debug, RlpDecodable, RlpEncodable, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct SignalInfo {
    location:  SignalLocation,
    arg_count: U256,
    slot_list: Vec::<Slot>,
}

impl SignalInfo {
    // Return a fresh SignalInfo.
    pub fn new(owner: &Address, signal_key: &[u8], arg_count: &U256) -> Self {
        let new = SignalInfo {
            location:  SignalLocation::new(owner, signal_key),
            arg_count: arg_count.clone(),
            slot_list: Vec::new(),
        };
        new
    }

    // Bind a slot to this signal.
    pub fn add_to_slot_list(&mut self, slot_info: &SlotInfo) {
        let slot = Slot::new(slot_info);
        self.slot_list.push(slot);
    }

    // Removes a slot given a location.
    pub fn remove_from_slot_list(&mut self, loc: &SlotLocation) {
        for i in 0..self.slot_list.clone().len() {
            let slot = &self.slot_list[i];
            if slot.location().address() == loc.address() && slot.location().slot_key() == loc.slot_key() {
                self.slot_list.remove(i);
            }
        }
    }

    // Getters
    pub fn location(&self) -> &SignalLocation {
        &self.location
    }
    pub fn arg_count(&self) -> &U256 {
        &self.arg_count
    }
    pub fn slot_list(&self) -> &Vec::<Slot> {
        &self.slot_list
    }
}

// SlotInfo. Holds the information that the owner of the slot needs maintain.
// Whereas Slot is maintained by the owner of the signal that we binded to,
// SlotInfo is owned by the owner contract who implements the handler. As a
// result a few things are different, most notably, we need to keep a list
// of the signals this slot is binded to.
#[derive(
    Clone, Debug, RlpDecodable, RlpEncodable, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct SlotInfo {
    // Location on the network. Used to identify this slot uniquely.
    location: SlotLocation,
    // Number of arguments expected from a binded signal
    arg_count: U256,
    // Gas limit for slot execution.
    gas_limit: U256,
    // Gas ratio for slot execution.
    gas_ratio_numerator: U256,
    gas_ratio_denominator: U256,
    // List of keys to the signals that this slot is binded to.
    // This may not be neccessary for functionality, but might be
    // useful down the road when implementing automatic cleanup.
    bind_list: Vec::<SignalLocation>,
}

impl SlotInfo {
    // Create a new SlotInfo.
    pub fn new(
        owner: &Address, contract: &Address, slot_key: &[u8], arg_count: &U256,
        gas_limit: &U256, numerator: &U256, denominator: &U256
    ) -> Self {
        let loc = SlotLocation::new(owner, contract, slot_key);
        let new = SlotInfo {
            location:              loc,
            arg_count:             arg_count.clone(),
            gas_limit:             gas_limit.clone(),
            gas_ratio_numerator:   numerator.clone(),
            gas_ratio_denominator: denominator.clone(),
            bind_list:             Vec::new(),
        };
        new
    }
    // Add a signal to the bind list.
    pub fn add_to_bind_list(&mut self, loc: &SignalLocation) {
        let loc = loc.clone();
        self.bind_list.push(loc);
    }
    // Remove a signal from the bind list.
    pub fn remove_from_bind_list(&mut self, loc: &SignalLocation) {
        for i in 0..self.bind_list.clone().len() {
            let sig = &self.bind_list[i];
            if sig.address() == loc.address() && sig.signal_key() == loc.signal_key() {
                self.bind_list.remove(i);
            }
        }
    }

    // Getters
    pub fn location(&self) -> &SlotLocation {
        &self.location
    }
    pub fn arg_count(&self) -> &U256 {
        &self.arg_count
    }
    pub fn gas_limit(&self) -> &U256 {
        &self.gas_limit
    }
    pub fn gas_ratio_numerator(&self) -> &U256 {
        &self.gas_ratio_numerator
    }
    pub fn gas_ratio_denominator(&self) -> &U256 {
        &self.gas_ratio_denominator
    }
    pub fn bind_list(&self) -> &Vec<SignalLocation> {
        &self.bind_list
    }
}

// Slot. Holds the information that the signal needs to maintain. Helps in the creation of
// construction of a Slot Transaction upon the emission of a signal. Although almost all
// information is derived from the SlotInfo, we need the address of the owner of the slot as
// well as a unique id to be provided. This id allows us to parse through the list of slots
// when we need to cleanup or delete entries.
#[derive(
    Clone, Debug, RlpDecodable, RlpEncodable, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Slot {
    // Address of contract that owns this slot.
    location: SlotLocation,
    // Gas limit for slot execution.
    gas_limit: U256,
    // Gas ratio for slot execution.
    gas_ratio_numerator: U256,
    gas_ratio_denominator: U256,
}

impl Slot {
    // Create a new slot out of a SlotInfo.
    pub fn new(slot_info: &SlotInfo) -> Self {
        let new = Slot {
            location:              slot_info.location.clone(),
            gas_limit:             slot_info.gas_limit.clone(),
            gas_ratio_numerator:   slot_info.gas_ratio_numerator.clone(),
            gas_ratio_denominator: slot_info.gas_ratio_denominator.clone(),
        };
        new
    }

    // Getters.
    pub fn location(&self) -> &SlotLocation {
        &self.location
    }
    pub fn gas_limit(&self) -> &U256 {
        &self.gas_limit
    }
    pub fn gas_ratio_numerator(&self) -> &U256 {
        &self.gas_ratio_numerator
    }
    pub fn gas_ratio_denominator(&self) -> &U256 {
        &self.gas_ratio_denominator
    }

    // Returns the method id of the slot
    pub fn get_method_id(&self) -> Bytes {
        self.location.slot_key()[0..4].to_vec()
    }
}

// SlotTx. Transactions that execute a slot. It holds a slot as well as the block number for execution and
// the a vector of arguments passed in by the signal.
#[derive(
    Clone, Debug, RlpDecodable, RlpEncodable, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct SlotTx {
    // Address of contract that owns this slot.
    location: SlotLocation,
    // Gas limit for slot execution.
    gas_limit: U256,
    // Gas ratio for slot execution.
    gas_ratio_numerator: U256,
    gas_ratio_denominator: U256,
    // Block number of when this transaction becomes available for execution.
    epoch_height: u64,
    // Vector of arguments emitted by the signal.
    argv: Bytes,
    //check data is fix or dynamic for abi encoding
    is_fix : bool,
    //the length of the data if dynamic
    data_length: Vec<u8>,
    // Gas price. Determined during packing.
    gas_price: U256,
    // Gas upfront cost.
    gas_upfront: U256,
}

impl SlotTx {
    pub fn new(
        slot: &Slot, epoch_height: &u64, argv: &Bytes,
        is_fix: bool, data_length: &Vec<u8>
    ) -> Self {
        let new = SlotTx {
            location:              slot.location().clone(),
            gas_limit:             slot.gas_limit.clone(),
            gas_ratio_numerator:   slot.gas_ratio_numerator().clone(),
            gas_ratio_denominator: slot.gas_ratio_denominator.clone(),
            epoch_height:          epoch_height.clone(),
            argv:                  argv.clone(),
            is_fix:                is_fix,
            data_length:           data_length.to_vec(),
            // Gas price is set when packed in the transaction pool.
            gas_price:             U256::zero(),
            gas_upfront:           U256::zero(),
        };
        new
    }

    // Getters
    pub fn location(&self) -> &SlotLocation {
        &self.location
    }
    pub fn address(&self) -> &Address {
        &self.location.address()
    }
    pub fn contract_address(&self) -> &Address {
        &self.location.contract_address()
    }
    pub fn slot_key(&self) -> &Bytes {
        &self.location.slot_key()
    }
    pub fn gas_limit(&self) -> &U256 {
        &self.gas_limit
    }
    pub fn gas_ratio_numerator(&self) -> &U256 {
        &self.gas_ratio_numerator
    }
    pub fn gas_ratio_denominator(&self) -> &U256 {
        &self.gas_ratio_denominator
    }
    pub fn epoch_height(&self) -> u64 {
        self.epoch_height
    }
    pub fn argv(&self) -> Bytes {
        self.argv.clone()
    }
    pub fn gas_price(&self) -> &U256 {
        &self.gas_price
    }
    pub fn gas_upfront(&self) -> &U256 {
        &self.gas_upfront
    }

    pub fn is_duplicated(&self, tx: &SlotTx) -> bool {
        self.location == *tx.location() && self.argv == tx.argv()
        && self.epoch_height == tx.epoch_height()
    }

    // Functions for ABI purposes. This becomes important when calling slot code.
    // The standard ABI protocol involves having each function assigned a method id.
    // To call a function, the 4 byte method ID is prepended to the argument/data vector.
    pub fn get_method_id(&self) -> Bytes {
        self.location.slot_key()[0..4].to_vec()
    }

    //encoding idea and assumption:
    /*BETTER to only accept bytes<M>, bytes, bytes<M>[N]
    bytes<M>: methed ID + (M bytes + padding zeros)
    bytes: methed ID + 0x0000..0020 + (padding zeros + datalength)+ 32bytes data + 32bytes data + .... + (Nbytes data + padding zeros) where N <= 32
    bytes<M>[N]: method ID + (bytes<M>[0] + padding zeros) + (bytes<M>[1] + padding zeros) +..+ (bytes<M>[N-1] + padding zeros)

    if uint, int, uint[], int[], uint<M>, int<M> where M is between 0 to 256 are accepted
    do the same thing above but padding zeros ahead of the data

    Update: the arguements should already be padded by zeros, don't care about zeros, only care about it is fixed or dynamic type
    */
    pub fn encode(&self) -> Bytes {
        let mut ret = self.get_method_id().clone();
        if self.is_fix {
            ret.extend_from_slice(&self.argv[..]);
        }else{
            let mut off_part = vec![0u8; 31];
            off_part.push(64);
            // let mut len_part = vec![0u8; 32];
            // len_part[31] = self.data_length;
            ret.extend_from_slice(&off_part[..]);
            ret.extend_from_slice(&self.data_length[..]);
            ret.extend_from_slice(&self.argv[..]);
        }
        ret
    }

    // The two functions below are called in the tx pool, when these transactions are getting packed.

    // Calculate gas price.
    pub fn calculate_and_set_gas_price(&mut self, average_gas_price: &U256) {
        self.gas_price = average_gas_price * self.gas_ratio_numerator / self.gas_ratio_denominator;
    }
    // Calculate gas upfront cost.
    pub fn set_gas_upfront(&mut self, gas_upfront: U256) {
        self.gas_upfront = gas_upfront;
    }

}

/* Signal and Slots end */
//////////////////////////////////////////////////////////////////////
