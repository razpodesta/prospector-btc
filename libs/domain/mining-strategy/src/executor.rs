// libs/domain/mining-strategy/src/executor.rs

use prospector_domain_models::work::{WorkOrder, SearchStrategy, ForensicTarget};
use prospector_core_probabilistic::filter_wrapper::RichListFilter;
use crate::combinatoric::CombinatoricIterator;
use crate::dictionary::DictionaryIterator;
use prospector_domain_forensics::DebianIterator; // <--- INTEGRACIÓN
use rayon::prelude::*;
use prospector_core_math::public_key::SafePublicKey;
use prospector_core_gen::address_legacy::pubkey_to_address;
use prospector_core_math::private_key::SafePrivateKey;

pub trait FindingHandler: Sync + Send {
    fn on_finding(&self, address: String, pk: SafePrivateKey, source: String);
}

pub struct StrategyExecutor;

impl StrategyExecutor {
    pub fn execute<H: FindingHandler>(
        job: &WorkOrder,
        filter: &RichListFilter,
        context_data: &ExecutorContext,
        handler: &H
    ) {
        match &job.strategy {
            SearchStrategy::Combinatoric { prefix, suffix, start_index, end_index } => {
                let iter = CombinatoricIterator::new(*start_index, *end_index, prefix.clone(), suffix.clone());
                iter.par_bridge().for_each(|(phrase, pk)| {
                    Self::check_candidate(filter, pk, format!("comb:{}", phrase), handler);
                });
            },

            SearchStrategy::Dictionary { dataset_url: _, limit } => {
                if let Some(words) = &context_data.dictionary_cache {
                    let iter = DictionaryIterator::new(words, *limit);
                    iter.par_bridge().for_each(|(phrase, pk)| {
                        Self::check_candidate(filter, pk, format!("dict:{}", phrase), handler);
                    });
                }
            },

            SearchStrategy::ForensicScan { target, range_start, range_end } => {
                match target {
                    ForensicTarget::DebianOpenSSL => {
                        // Rango típico de PIDs: 0 a 32768
                        let iter = DebianIterator::new(*range_start, *range_end);
                        iter.par_bridge().for_each(|(source, pk)| {
                            Self::check_candidate(filter, pk, source, handler);
                        });
                    },
                    ForensicTarget::AndroidSecureRandom => {
                        // TODO: Implementar Android PRNG Iterator
                        // Requiere lógica de PRNG Java seedada
                    }
                }
            },

            SearchStrategy::Random { .. } => {
                // Implementación futura
            }
        }
    }

    #[inline(always)]
    fn check_candidate<H: FindingHandler>(
        filter: &RichListFilter,
        pk: SafePrivateKey,
        source: String,
        handler: &H
    ) {
        let pub_key = SafePublicKey::from_private(&pk);
        // La mayoría de monedas perdidas forenses son Legacy (uncompressed)
        let addr = pubkey_to_address(&pub_key, false);

        if filter.contains(&addr) {
            handler.on_finding(addr, pk, source);
        }
    }
}

#[derive(Default)]
pub struct ExecutorContext {
    pub dictionary_cache: Option<Vec<String>>,
}
