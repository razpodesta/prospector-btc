use prospector_domain_models::work::{WorkOrder, SearchStrategy};
use prospector_core_probabilistic::filter_wrapper::RichListFilter;
use crate::{BrainwalletIterator, CombinatoricIterator};
use rayon::prelude::*;
use prospector_core_math::public_key::SafePublicKey;
use prospector_core_gen::address_legacy::pubkey_to_address;

pub trait StrategyExecutor {
    fn execute<F>(&self, filter: &RichListFilter, on_find: F)
    where F: Fn(String, prospector_core_math::private_key::SafePrivateKey) + Sync + Send;
}

impl StrategyExecutor for WorkOrder {
    fn execute<F>(&self, filter: &RichListFilter, on_find: F)
    where F: Fn(String, prospector_core_math::private_key::SafePrivateKey) + Sync + Send
    {
        match &self.strategy {
             SearchStrategy::Combinatoric { prefix, suffix, start_index, end_index } => {
                let iter = CombinatoricIterator::new(*start_index, *end_index, prefix.clone(), suffix.clone());
                iter.par_bridge().for_each(|(phrase, pk)| {
                    // Lógica de chequeo rápida
                    let pub_key = SafePublicKey::from_private(&pk);
                    let addr = pubkey_to_address(&pub_key, false);
                    if filter.contains(&addr) {
                        on_find(format!("comb:{}", phrase), pk);
                    }
                });
             },
             // ... Implementar otros casos
             _ => {}
        }
    }
}
