use std::{collections::BTreeSet, net::SocketAddr};

use bincode::{serialize, Error as BincodeError};
use bv::BitVec;
use serde::Serialize as SerdeSerialize;
use serde_derive::{Deserialize, Serialize};

use solana_bloom::bloom::Bloom;
use solana_sdk::{
  hash::{self, Hash},
  pubkey::Pubkey,
  signature::{Keypair, Signature, Signer},
  transaction::Transaction,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyContactInfo {
  pub id: Pubkey,
  /// gossip address
  pub gossip: SocketAddr,
  /// address to connect to for replication
  pub tvu: SocketAddr,
  /// address to forward shreds to
  pub tvu_forwards: SocketAddr,
  /// address to send repair responses to
  pub repair: SocketAddr,
  /// transactions address
  pub tpu: SocketAddr,
  /// address to forward unprocessed transactions to
  pub tpu_forwards: SocketAddr,
  /// address to which to send bank state requests
  pub tpu_vote: SocketAddr,
  /// address to which to send JSON-RPC requests
  pub rpc: SocketAddr,
  /// websocket for JSON-RPC push notifications
  pub rpc_pubsub: SocketAddr,
  /// address to send repair requests to
  pub serve_repair: SocketAddr,
  /// latest wallclock picked
  pub wallclock: u64,
  /// node shred version
  pub shred_version: u16,
}

#[macro_export]
macro_rules! socketaddr_default {
  () => {
    std::net::SocketAddr::from((std::net::Ipv4Addr::from(0), 0))
  };
}

impl Default for LegacyContactInfo {
  fn default() -> Self {
    LegacyContactInfo {
      id: Pubkey::default(),
      gossip: socketaddr_default!(),
      tvu: socketaddr_default!(),
      tvu_forwards: socketaddr_default!(),
      repair: socketaddr_default!(),
      tpu: socketaddr_default!(),
      tpu_forwards: socketaddr_default!(),
      tpu_vote: socketaddr_default!(),
      rpc: socketaddr_default!(),
      rpc_pubsub: socketaddr_default!(),
      serve_repair: socketaddr_default!(),
      wallclock: 0,
      shred_version: 0,
    }
  }
}

pub type VoteIndex = u8;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Vote {
  pub(crate) from: Pubkey,
  transaction: Transaction,
  pub(crate) wallclock: u64,
}

pub type Slot = u64;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SnapshotHashes {
  pub from: Pubkey,
  pub hashes: Vec<(Slot, Hash)>,
  pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyVersion1 {
  major: u16,
  minor: u16,
  patch: u16,
  commit: Option<u32>, // first 4 bytes of the sha1 commit hash
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyVersion {
  pub from: Pubkey,
  pub wallclock: u64,
  pub version: LegacyVersion1,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyVersion2 {
  pub major: u16,
  pub minor: u16,
  pub patch: u16,
  pub commit: Option<u32>,
  pub feature_set: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Version {
  pub from: Pubkey,
  pub wallclock: u64,
  pub version: LegacyVersion2,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct NodeInstance {
  pub from: Pubkey,
  pub wallclock: u64,
  pub timestamp: u64,
  pub token: u64,
}

pub type EpochSlotsIndex = u8;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Flate2 {
  pub first_slot: Slot,
  pub num: usize,
  pub compressed: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Uncompressed {
  pub first_slot: Slot,
  pub num: usize,
  pub slots: BitVec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum CompressedSlots {
  Flate2(Flate2),
  Uncompressed(Uncompressed),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct EpochSlots {
  pub from: Pubkey,
  pub slots: Vec<CompressedSlots>,
  pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
enum DeprecatedCompressionType {
  Uncompressed,
  GZip,
  BZip2,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct DeprecatedEpochIncompleteSlots {
  first: Slot,
  compression: DeprecatedCompressionType,
  compressed_list: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LowestSlot {
  pub from: Pubkey,
  root: Slot,
  pub lowest: Slot,
  slots: BTreeSet<Slot>,
  stash: Vec<DeprecatedEpochIncompleteSlots>,
  pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct IncrementalSnapshotHashes {
  pub from: Pubkey,
  pub base: (Slot, Hash),
  pub hashes: Vec<(Slot, Hash)>,
  pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum CrdsData {
  LegacyContactInfo(LegacyContactInfo),    // OK len:254
  Vote(VoteIndex, Vote),                   // OK len:472
  LowestSlot(u8, LowestSlot),              // OK len:185
  SnapshotHashes(SnapshotHashes),          // OK len:240
  AccountsHashes(SnapshotHashes),          // OK len:800
  EpochSlots(EpochSlotsIndex, EpochSlots), // OK len:1049
  LegacyVersion(LegacyVersion),            // OK len:163
  Version(Version),                        // OK len:167
  NodeInstance(NodeInstance),              // OK len:168
  DuplicateShred(),                        // ??
  IncrementalSnapshotHashes(IncrementalSnapshotHashes), // OK len:360
  ContactInfo(),                           // ??
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct CrdsValue {
  pub signature: Signature,
  pub data: CrdsData,
}

impl CrdsValue {
  pub fn new_signed(data: CrdsData, keypair: &Keypair) -> Self {
    let signable_data = serialize(&data).expect("failed to serialize CrdsData");
    let signature = keypair.sign_message(&signable_data);
    Self { signature, data }
  }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct CrdsFilter {
  pub filter: Bloom<Hash>,
  pub mask: u64,
  pub mask_bits: u32,
}

impl Default for CrdsFilter {
  fn default() -> Self {
    fn compute_mask(seed: u64, mask_bits: u32) -> u64 {
      assert!(seed <= 2u64.pow(mask_bits));
      let seed: u64 = seed.checked_shl(64 - mask_bits).unwrap_or(0x0);
      seed | (!0u64).checked_shr(mask_bits).unwrap_or(!0x0)
    }
    fn mask_bits(num_items: f64, max_items: f64) -> u32 {
      // for small ratios this can result in a negative number, ensure it returns 0 instead
      ((num_items / max_items).log2().ceil()).max(0.0) as u32
    }

    let max_items = 1287f64;
    const FALSE_RATE: f64 = 0.1f64;
    let max_bits = 7424u32;
    let num_items: usize = 512;
    let mask_bits = mask_bits(num_items as f64, max_items);

    let bloom: Bloom<Hash> = Bloom::random(max_items as usize, FALSE_RATE, max_bits as usize);

    CrdsFilter {
      filter: bloom,
      mask: compute_mask(0_u64, mask_bits),
      mask_bits,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PingGeneric<T> {
  from: Pubkey,
  token: T,
  signature: Signature,
}

/// Number of bytes in the randomly generated token sent with ping messages.
const GOSSIP_PING_TOKEN_SIZE: usize = 32;

pub type Ping = PingGeneric<[u8; GOSSIP_PING_TOKEN_SIZE]>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Pong {
  from: Pubkey,
  hash: Hash, // Hash of received ping token.
  signature: Signature,
}

const PING_PONG_HASH_PREFIX: &[u8] = "SOLANA_PING_PONG".as_bytes();

impl Pong {
  pub fn new<T: SerdeSerialize>(
    ping: &PingGeneric<T>,
    keypair: &Keypair,
  ) -> Result<Self, BincodeError> {
    let token = serialize(&ping.token)?;
    let hash = hash::hashv(&[PING_PONG_HASH_PREFIX, &token]);
    let pong = Pong {
      from: keypair.pubkey(),
      hash,
      signature: keypair.sign_message(hash.as_ref()),
    };
    Ok(pong)
  }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Protocol {
  PullRequest(CrdsFilter, CrdsValue),
  PullResponse(Pubkey, Vec<CrdsValue>),
  PushMessage(Pubkey, Vec<CrdsValue>),
  PruneMessage(Pubkey),
  PingMessage(Ping),
  PongMessage(Pong),
}

//tests
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_serialize() {
    let keypair = Keypair::new();

    //let crds_data = CrdsData::Vote();
    let crds_data = CrdsData::LegacyContactInfo(LegacyContactInfo::default());
    let crds_value = CrdsValue::new_signed(crds_data.clone(), &keypair);
    println!("crds_value: {:?}", crds_value);
  }

  #[test]
  fn test_crds_filter() {
    let crds_filter = CrdsFilter::default();
    println!("crds_filter: {:?}", crds_filter);
  }
}
