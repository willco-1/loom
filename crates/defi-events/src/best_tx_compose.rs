use alloy_primitives::U256;

use crate::TxComposeData;

pub struct BestTxCompose {
    validity_pct: Option<U256>,
    best_profit_swap: Option<TxComposeData>,
    best_profit_gas_ratio_swap: Option<TxComposeData>,
    best_tips_swap: Option<TxComposeData>,
    best_tips_gas_ratio_swap: Option<TxComposeData>,

}

impl Default for BestTxCompose {
    fn default() -> Self {
        Self {
            validity_pct: None,
            best_profit_swap: None,
            best_profit_gas_ratio_swap: None,
            best_tips_swap: None,
            best_tips_gas_ratio_swap: None,
        }
    }
}

impl BestTxCompose {
    pub fn new_with_pct<T: Into<U256>>(validity_pct: T) -> Self {
        BestTxCompose {
            validity_pct: Some(validity_pct.into()),
            ..Default::default()
        }
    }

    pub fn check(&mut self, request: &TxComposeData) -> bool {
        let mut is_ok = false;

        match &self.best_profit_swap {
            None => {
                self.best_profit_swap = Some(request.clone());
                is_ok = true;
            }
            Some(best_swap) => {
                if best_swap.swap.abs_profit_eth() < request.swap.abs_profit_eth() {
                    self.best_profit_swap = Some(request.clone());
                    is_ok = true;
                } else {
                    match self.validity_pct {
                        Some(pct) => {
                            if (best_swap.swap.abs_profit_eth() * pct) / U256::from(10000) < request.swap.abs_profit_eth() {
                                is_ok = true
                            }
                        }
                        None => {}
                    }
                }
            }
        }

        if request.tips.is_some() {
            match &self.best_tips_swap {
                Some(best_swap) => {
                    if best_swap.tips.unwrap_or_default() < request.tips.unwrap_or_default() {
                        self.best_tips_swap = Some(request.clone());
                        is_ok = true;
                    } else {
                        match self.validity_pct {
                            Some(pct) => {
                                if (best_swap.tips.unwrap_or_default() * pct) / U256::from(10000) < request.tips.unwrap_or_default() {
                                    is_ok = true
                                }
                            }
                            None => {}
                        }
                    }
                }
                None => {
                    self.best_tips_swap = Some(request.clone());
                    is_ok = true;
                }
            }
        }


        if request.gas != 0 {
            match &self.best_tips_gas_ratio_swap {
                Some(best_swap) => {
                    if best_swap.tips_gas_ratio() < request.tips_gas_ratio() {
                        self.best_tips_gas_ratio_swap = Some(request.clone());
                        is_ok = true;
                    } else {
                        match self.validity_pct {
                            Some(pct) => {
                                if (best_swap.tips_gas_ratio() * pct) / U256::from(10000) < request.tips_gas_ratio() {
                                    is_ok = true
                                }
                            }
                            None => {}
                        }
                    }
                }
                None => {
                    self.best_tips_gas_ratio_swap = Some(request.clone());
                    is_ok = true;
                }
            }

            match &self.best_profit_gas_ratio_swap {
                Some(best_swap) => {
                    if best_swap.profit_eth_gas_ratio() < request.profit_eth_gas_ratio() {
                        self.best_profit_gas_ratio_swap = Some(request.clone());
                        is_ok = true;
                    } else {
                        match self.validity_pct {
                            Some(pct) => {
                                if (best_swap.profit_eth_gas_ratio() * pct) / U256::from(10000) < request.profit_eth_gas_ratio() {
                                    is_ok = true
                                }
                            }
                            None => {}
                        }
                    }
                }
                None => {
                    self.best_profit_gas_ratio_swap = Some(request.clone());
                    is_ok = true;
                }
            }
        }
        is_ok
    }
}
