#![allow(non_upper_case_globals)]
mod types;
mod utils;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use api_models::{
    admin as admin_api, conditional_configs::ConditionalConfigs, enums as api_model_enums,
    routing::ConnectorSelection, surcharge_decision_configs::SurchargeDecisionConfigs,
};
use common_enums::RoutableConnectors;
use connector_configs::{
    common_config::{ConnectorApiIntegrationPayload, DashboardRequestPayload},
    connector,
};
use currency_conversion::{
    conversion::convert as convert_currency, types as currency_conversion_types,
};
use euclid::{
    backend::{inputs, interpreter::InterpreterBackend, EuclidBackend},
    dssa::{self, analyzer, graph::CgraphExt, state_machine, truth},
    frontend::{
        ast,
        dir::{self, enums as dir_enums, EuclidDirFilter},
    },
};
use once_cell::sync::OnceCell;
use strum::{EnumMessage, EnumProperty, VariantNames};
use wasm_bindgen::prelude::*;

use crate::utils::JsResultExt;
type JsResult = Result<JsValue, JsValue>;

struct SeedData<'a> {
    cgraph: hyperswitch_constraint_graph::ConstraintGraph<'a, dir::DirValue>,
    connectors: Vec<ast::ConnectorChoice>,
}

static SEED_DATA: OnceCell<SeedData<'_>> = OnceCell::new();
static SEED_FOREX: OnceCell<currency_conversion_types::ExchangeRates> = OnceCell::new();

/// This function can be used by the frontend to educate wasm about the forex rates data.
/// The input argument is a struct fields base_currency and conversion where later is all the conversions associated with the base_currency
/// to all different currencies present.
#[wasm_bindgen(js_name = setForexData)]
pub fn seed_forex(forex: JsValue) -> JsResult {
    let forex: currency_conversion_types::ExchangeRates = serde_wasm_bindgen::from_value(forex)?;
    SEED_FOREX
        .set(forex)
        .map_err(|_| "Forex has already been seeded".to_string())
        .err_to_js()?;

    Ok(JsValue::NULL)
}

/// This function can be used to perform currency_conversion on the input amount, from_currency,
/// to_currency which are all expected to be one of currencies we already have in our Currency
/// enum.
#[wasm_bindgen(js_name = convertCurrency)]
pub fn convert_forex_value(amount: i64, from_currency: JsValue, to_currency: JsValue) -> JsResult {
    let forex_data = SEED_FOREX
        .get()
        .ok_or("Forex Data not seeded")
        .err_to_js()?;
    let from_currency: common_enums::Currency = serde_wasm_bindgen::from_value(from_currency)?;
    let to_currency: common_enums::Currency = serde_wasm_bindgen::from_value(to_currency)?;
    let converted_amount = convert_currency(forex_data, from_currency, to_currency, amount)
        .map_err(|_| "conversion not possible for provided values")
        .err_to_js()?;

    Ok(serde_wasm_bindgen::to_value(&converted_amount)?)
}

/// This function can be used by the frontend to provide the WASM with information about
/// all the merchant's connector accounts. The input argument is a vector of all the merchant's
/// connector accounts from the API.
#[wasm_bindgen(js_name = seedKnowledgeGraph)]
pub fn seed_knowledge_graph(mcas: JsValue) -> JsResult {
    let mcas: Vec<admin_api::MerchantConnectorResponse> = serde_wasm_bindgen::from_value(mcas)?;
    let connectors: Vec<ast::ConnectorChoice> = mcas
        .iter()
        .map(|mca| {
            Ok::<_, strum::ParseError>(ast::ConnectorChoice {
                connector: RoutableConnectors::from_str(&mca.connector_name)?,
                #[cfg(not(feature = "connector_choice_mca_id"))]
                sub_label: mca.business_sub_label.clone(),
            })
        })
        .collect::<Result<_, _>>()
        .map_err(|_| "invalid connector name received")
        .err_to_js()?;
    let pm_filter = kgraph_utils::types::PaymentMethodFilters(HashMap::new());
    let config = kgraph_utils::types::CountryCurrencyFilter {
        connector_configs: HashMap::new(),
        default_configs: Some(pm_filter),
    };
    let mca_graph = kgraph_utils::mca::make_mca_graph(mcas, &config).err_to_js()?;
    let analysis_graph =
        hyperswitch_constraint_graph::ConstraintGraph::combine(&mca_graph, &truth::ANALYSIS_GRAPH)
            .err_to_js()?;

    SEED_DATA
        .set(SeedData {
            cgraph: analysis_graph,
            connectors,
        })
        .map_err(|_| "Knowledge Graph has been already seeded".to_string())
        .err_to_js()?;

    Ok(JsValue::NULL)
}

/// This function allows the frontend to get all the merchant's configured
/// connectors that are valid for a rule based on the conditions specified in
/// the rule
#[wasm_bindgen(js_name = getValidConnectorsForRule)]
pub fn get_valid_connectors_for_rule(rule: JsValue) -> JsResult {
    let seed_data = SEED_DATA.get().ok_or("Data not seeded").err_to_js()?;

    let rule: ast::Rule<ConnectorSelection> = serde_wasm_bindgen::from_value(rule)?;
    let dir_rule = ast::lowering::lower_rule(rule).err_to_js()?;
    let mut valid_connectors: Vec<(ast::ConnectorChoice, dir::DirValue)> = seed_data
        .connectors
        .iter()
        .cloned()
        .map(|choice| (choice.clone(), dir::DirValue::Connector(Box::new(choice))))
        .collect();
    let mut invalid_connectors: HashSet<ast::ConnectorChoice> = HashSet::new();

    let mut ctx_manager = state_machine::RuleContextManager::new(&dir_rule, &[]);

    let dummy_meta = HashMap::new();

    // For every conjunctive context in the Rule, verify validity of all still-valid connectors
    // using the knowledge graph
    while let Some(ctx) = ctx_manager.advance_mut().err_to_js()? {
        // Standalone conjunctive context analysis to ensure the context itself is valid before
        // checking it against merchant's connectors
        seed_data
            .cgraph
            .perform_context_analysis(
                ctx,
                &mut hyperswitch_constraint_graph::Memoization::new(),
                None,
            )
            .err_to_js()?;

        // Update conjunctive context and run analysis on all of merchant's connectors.
        for (conn, choice) in &valid_connectors {
            if invalid_connectors.contains(conn) {
                continue;
            }

            let ctx_val = dssa::types::ContextValue::assertion(choice, &dummy_meta);
            ctx.push(ctx_val);
            let analysis_result = seed_data.cgraph.perform_context_analysis(
                ctx,
                &mut hyperswitch_constraint_graph::Memoization::new(),
                None,
            );
            if analysis_result.is_err() {
                invalid_connectors.insert(conn.clone());
            }
            ctx.pop();
        }
    }

    valid_connectors.retain(|(k, _)| !invalid_connectors.contains(k));

    let valid_connectors: Vec<ast::ConnectorChoice> =
        valid_connectors.into_iter().map(|c| c.0).collect();

    Ok(serde_wasm_bindgen::to_value(&valid_connectors)?)
}

#[wasm_bindgen(js_name = analyzeProgram)]
pub fn analyze_program(js_program: JsValue) -> JsResult {
    let program: ast::Program<ConnectorSelection> = serde_wasm_bindgen::from_value(js_program)?;
    analyzer::analyze(program, SEED_DATA.get().map(|sd| &sd.cgraph)).err_to_js()?;
    Ok(JsValue::NULL)
}

#[wasm_bindgen(js_name = runProgram)]
pub fn run_program(program: JsValue, input: JsValue) -> JsResult {
    let program: ast::Program<ConnectorSelection> = serde_wasm_bindgen::from_value(program)?;
    let input: inputs::BackendInput = serde_wasm_bindgen::from_value(input)?;

    let backend = InterpreterBackend::with_program(program).err_to_js()?;

    let res: euclid::backend::BackendOutput<ConnectorSelection> =
        backend.execute(input).err_to_js()?;

    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen(js_name = getAllConnectors)]
pub fn get_all_connectors() -> JsResult {
    Ok(serde_wasm_bindgen::to_value(RoutableConnectors::VARIANTS)?)
}

#[wasm_bindgen(js_name = getAllKeys)]
pub fn get_all_keys() -> JsResult {
    let keys: Vec<&'static str> = dir::DirKey::VARIANTS
        .iter()
        .copied()
        .filter(|s| s != &"Connector")
        .collect();
    Ok(serde_wasm_bindgen::to_value(&keys)?)
}

#[wasm_bindgen(js_name = getKeyType)]
pub fn get_key_type(key: &str) -> Result<String, String> {
    let key = dir::DirKey::from_str(key).map_err(|_| "Invalid key received".to_string())?;
    let key_str = key.get_type().to_string();
    Ok(key_str)
}

#[wasm_bindgen(js_name = getThreeDsKeys)]
pub fn get_three_ds_keys() -> JsResult {
    let keys = <ConditionalConfigs as EuclidDirFilter>::ALLOWED;
    Ok(serde_wasm_bindgen::to_value(keys)?)
}

#[wasm_bindgen(js_name= getSurchargeKeys)]
pub fn get_surcharge_keys() -> JsResult {
    let keys = <SurchargeDecisionConfigs as EuclidDirFilter>::ALLOWED;
    Ok(serde_wasm_bindgen::to_value(keys)?)
}

#[wasm_bindgen(js_name=parseToString)]
pub fn parser(val: String) -> String {
    ron_parser::my_parse(val)
}

#[wasm_bindgen(js_name = getVariantValues)]
pub fn get_variant_values(key: &str) -> Result<JsValue, JsValue> {
    let key = dir::DirKey::from_str(key).map_err(|_| "Invalid key received".to_string())?;

    let variants: &[&str] = match key {
        dir::DirKey::PaymentMethod => dir_enums::PaymentMethod::VARIANTS,
        dir::DirKey::CardType => dir_enums::CardType::VARIANTS,
        dir::DirKey::CardNetwork => dir_enums::CardNetwork::VARIANTS,
        dir::DirKey::PayLaterType => dir_enums::PayLaterType::VARIANTS,
        dir::DirKey::WalletType => dir_enums::WalletType::VARIANTS,
        dir::DirKey::BankRedirectType => dir_enums::BankRedirectType::VARIANTS,
        dir::DirKey::CryptoType => dir_enums::CryptoType::VARIANTS,
        dir::DirKey::RewardType => dir_enums::RewardType::VARIANTS,
        dir::DirKey::AuthenticationType => dir_enums::AuthenticationType::VARIANTS,
        dir::DirKey::CaptureMethod => dir_enums::CaptureMethod::VARIANTS,
        dir::DirKey::PaymentCurrency => dir_enums::PaymentCurrency::VARIANTS,
        dir::DirKey::BusinessCountry => dir_enums::Country::VARIANTS,
        dir::DirKey::BillingCountry => dir_enums::Country::VARIANTS,
        dir::DirKey::BankTransferType => dir_enums::BankTransferType::VARIANTS,
        dir::DirKey::UpiType => dir_enums::UpiType::VARIANTS,
        dir::DirKey::SetupFutureUsage => dir_enums::SetupFutureUsage::VARIANTS,
        dir::DirKey::PaymentType => dir_enums::PaymentType::VARIANTS,
        dir::DirKey::MandateType => dir_enums::MandateType::VARIANTS,
        dir::DirKey::MandateAcceptanceType => dir_enums::MandateAcceptanceType::VARIANTS,
        dir::DirKey::CardRedirectType => dir_enums::CardRedirectType::VARIANTS,
        dir::DirKey::GiftCardType => dir_enums::GiftCardType::VARIANTS,
        dir::DirKey::VoucherType => dir_enums::VoucherType::VARIANTS,
        dir::DirKey::BankDebitType => dir_enums::BankDebitType::VARIANTS,

        dir::DirKey::PaymentAmount
        | dir::DirKey::Connector
        | dir::DirKey::CardBin
        | dir::DirKey::BusinessLabel
        | dir::DirKey::MetaData => Err("Key does not have variants".to_string())?,
    };

    Ok(serde_wasm_bindgen::to_value(variants)?)
}

#[wasm_bindgen(js_name = addTwo)]
pub fn add_two(n1: i64, n2: i64) -> i64 {
    n1 + n2
}

#[wasm_bindgen(js_name = getDescriptionCategory)]
pub fn get_description_category() -> JsResult {
    let keys = dir::DirKey::VARIANTS
        .iter()
        .copied()
        .filter(|s| s != &"Connector")
        .collect::<Vec<&'static str>>();
    let mut category: HashMap<Option<&str>, Vec<types::Details<'_>>> = HashMap::new();
    for key in keys {
        let dir_key = dir::DirKey::from_str(key).map_err(|_| "Invalid key received".to_string())?;
        let details = types::Details {
            description: dir_key.get_detailed_message(),
            kind: dir_key.clone(),
        };
        category
            .entry(dir_key.get_str("Category"))
            .and_modify(|val| val.push(details.clone()))
            .or_insert(vec![details]);
    }

    Ok(serde_wasm_bindgen::to_value(&category)?)
}

#[wasm_bindgen(js_name = getConnectorConfig)]
pub fn get_connector_config(key: &str) -> JsResult {
    let key = api_model_enums::Connector::from_str(key)
        .map_err(|_| "Invalid key received".to_string())?;
    let res = connector::ConnectorConfig::get_connector_config(key)?;
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[cfg(feature = "payouts")]
#[wasm_bindgen(js_name = getPayoutConnectorConfig)]
pub fn get_payout_connector_config(key: &str) -> JsResult {
    let key = api_model_enums::PayoutConnectors::from_str(key)
        .map_err(|_| "Invalid key received".to_string())?;
    let res = connector::ConnectorConfig::get_payout_connector_config(key)?;
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen(js_name = getAuthenticationConnectorConfig)]
pub fn get_authentication_connector_config(key: &str) -> JsResult {
    let key = api_model_enums::AuthenticationConnectors::from_str(key)
        .map_err(|_| "Invalid key received".to_string())?;
    let res = connector::ConnectorConfig::get_authentication_connector_config(key)?;
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen(js_name = getRequestPayload)]
pub fn get_request_payload(input: JsValue, response: JsValue) -> JsResult {
    let input: DashboardRequestPayload = serde_wasm_bindgen::from_value(input)?;
    let api_response: ConnectorApiIntegrationPayload = serde_wasm_bindgen::from_value(response)?;
    let result = DashboardRequestPayload::create_connector_request(input, api_response);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[wasm_bindgen(js_name = getResponsePayload)]
pub fn get_response_payload(input: JsValue) -> JsResult {
    let input: ConnectorApiIntegrationPayload = serde_wasm_bindgen::from_value(input)?;
    let result = ConnectorApiIntegrationPayload::get_transformed_response_payload(input);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[cfg(feature = "payouts")]
#[wasm_bindgen(js_name = getAllPayoutKeys)]
pub fn get_all_payout_keys() -> JsResult {
    let keys: Vec<&'static str> = dir::PayoutDirKeyKind::VARIANTS.to_vec();
    Ok(serde_wasm_bindgen::to_value(&keys)?)
}

#[cfg(feature = "payouts")]
#[wasm_bindgen(js_name = getPayoutVariantValues)]
pub fn get_payout_variant_values(key: &str) -> Result<JsValue, JsValue> {
    let key =
        dir::PayoutDirKeyKind::from_str(key).map_err(|_| "Invalid key received".to_string())?;

    let variants: &[&str] = match key {
        dir::PayoutDirKeyKind::BusinessCountry => dir_enums::BusinessCountry::VARIANTS,
        dir::PayoutDirKeyKind::BillingCountry => dir_enums::BillingCountry::VARIANTS,
        dir::PayoutDirKeyKind::PayoutType => dir_enums::PayoutType::VARIANTS,
        dir::PayoutDirKeyKind::WalletType => dir_enums::PayoutWalletType::VARIANTS,
        dir::PayoutDirKeyKind::BankTransferType => dir_enums::PayoutBankTransferType::VARIANTS,

        dir::PayoutDirKeyKind::PayoutAmount | dir::PayoutDirKeyKind::BusinessLabel => {
            Err("Key does not have variants".to_string())?
        }
    };

    Ok(serde_wasm_bindgen::to_value(variants)?)
}

#[cfg(feature = "payouts")]
#[wasm_bindgen(js_name = getPayoutDescriptionCategory)]
pub fn get_payout_description_category() -> JsResult {
    let keys = dir::PayoutDirKeyKind::VARIANTS.to_vec();
    let mut category: HashMap<Option<&str>, Vec<types::PayoutDetails<'_>>> = HashMap::new();
    for key in keys {
        let dir_key =
            dir::PayoutDirKeyKind::from_str(key).map_err(|_| "Invalid key received".to_string())?;
        let details = types::PayoutDetails {
            description: dir_key.get_detailed_message(),
            kind: dir_key.clone(),
        };
        category
            .entry(dir_key.get_str("Category"))
            .and_modify(|val| val.push(details.clone()))
            .or_insert(vec![details]);
    }

    Ok(serde_wasm_bindgen::to_value(&category)?)
}
