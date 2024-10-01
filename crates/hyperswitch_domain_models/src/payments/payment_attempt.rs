#[cfg(all(feature = "v1", feature = "olap"))]
use api_models::enums::Connector;
use common_enums as storage_enums;
use common_utils::{
    errors::{CustomResult, ValidationError},
    id_type, pii,
    types::{
        keymanager::{self, KeyManagerState},
        MinorUnit,
    },
};
use diesel_models::{
    PaymentAttempt as DieselPaymentAttempt, PaymentAttemptNew as DieselPaymentAttemptNew,
};
use error_stack::ResultExt;
use masking::Secret;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

#[cfg(feature = "v2")]
use masking::PeekInterface;

#[cfg(all(feature = "v1", feature = "olap"))]
use super::PaymentIntent;
#[cfg(feature = "v2")]
use crate::merchant_key_store::MerchantKeyStore;
use crate::{
    behaviour, errors,
    mandates::{MandateDataType, MandateDetails},
    ForeignIDRef,
};

#[async_trait::async_trait]
pub trait PaymentAttemptInterface {
    #[cfg(feature = "v1")]
    async fn insert_payment_attempt(
        &self,
        payment_attempt: PaymentAttemptNew,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v2")]
    async fn insert_payment_attempt(
        &self,
        key_manager_state: &KeyManagerState,
        merchant_key_store: &MerchantKeyStore,
        payment_attempt: PaymentAttempt,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn update_payment_attempt_with_attempt_id(
        &self,
        this: PaymentAttempt,
        payment_attempt: PaymentAttemptUpdate,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v2")]
    async fn update_payment_attempt_with_attempt_id(
        &self,
        key_manager_state: &KeyManagerState,
        merchant_key_store: &MerchantKeyStore,
        this: PaymentAttempt,
        payment_attempt: PaymentAttemptUpdate,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_by_connector_transaction_id_payment_id_merchant_id(
        &self,
        connector_transaction_id: &str,
        payment_id: &id_type::PaymentId,
        merchant_id: &id_type::MerchantId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_last_successful_attempt_by_payment_id_merchant_id(
        &self,
        payment_id: &id_type::PaymentId,
        merchant_id: &id_type::MerchantId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_last_successful_or_partially_captured_attempt_by_payment_id_merchant_id(
        &self,
        payment_id: &id_type::PaymentId,
        merchant_id: &id_type::MerchantId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_by_merchant_id_connector_txn_id(
        &self,
        merchant_id: &id_type::MerchantId,
        connector_txn_id: &str,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_by_payment_id_merchant_id_attempt_id(
        &self,
        payment_id: &id_type::PaymentId,
        merchant_id: &id_type::MerchantId,
        attempt_id: &str,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_by_attempt_id_merchant_id(
        &self,
        attempt_id: &str,
        merchant_id: &id_type::MerchantId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v2")]
    async fn find_payment_attempt_by_id(
        &self,
        key_manager_state: &KeyManagerState,
        merchant_key_store: &MerchantKeyStore,
        attempt_id: &str,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_payment_attempt_by_preprocessing_id_merchant_id(
        &self,
        preprocessing_id: &str,
        merchant_id: &id_type::MerchantId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentAttempt, errors::StorageError>;

    #[cfg(feature = "v1")]
    async fn find_attempts_by_merchant_id_payment_id(
        &self,
        merchant_id: &id_type::MerchantId,
        payment_id: &id_type::PaymentId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<Vec<PaymentAttempt>, errors::StorageError>;

    #[cfg(all(feature = "v1", feature = "olap"))]
    async fn get_filters_for_payments(
        &self,
        pi: &[PaymentIntent],
        merchant_id: &id_type::MerchantId,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<PaymentListFilters, errors::StorageError>;

    #[cfg(all(feature = "v1", feature = "olap"))]
    #[allow(clippy::too_many_arguments)]
    async fn get_total_count_of_filtered_payment_attempts(
        &self,
        merchant_id: &id_type::MerchantId,
        active_attempt_ids: &[String],
        connector: Option<Vec<Connector>>,
        payment_method: Option<Vec<storage_enums::PaymentMethod>>,
        payment_method_type: Option<Vec<storage_enums::PaymentMethodType>>,
        authentication_type: Option<Vec<storage_enums::AuthenticationType>>,
        merchant_connector_id: Option<Vec<id_type::MerchantConnectorAccountId>>,
        profile_id_list: Option<Vec<id_type::ProfileId>>,
        storage_scheme: storage_enums::MerchantStorageScheme,
    ) -> error_stack::Result<i64, errors::StorageError>;
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct AmountDetails {
    /// The total amount for this payment attempt. This includes all the surcharge and tax amounts.
    pub net_amount: MinorUnit,
    pub amount_to_capture: Option<MinorUnit>,
    pub surcharge_amount: Option<MinorUnit>,
    pub tax_on_surcharge: Option<MinorUnit>,
    pub amount_capturable: MinorUnit,
    pub shipping_cost: Option<MinorUnit>,
    pub order_tax_amount: Option<MinorUnit>,
}

#[cfg(feature = "v2")]
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct PaymentAttempt {
    pub payment_id: id_type::GlobalPaymentId,
    pub merchant_id: id_type::MerchantId,
    pub amount_details: AmountDetails,
    pub status: storage_enums::AttemptStatus,
    pub connector: Option<String>,
    pub error_message: Option<String>,
    pub authentication_type: Option<storage_enums::AuthenticationType>,
    pub created_at: PrimitiveDateTime,
    pub modified_at: PrimitiveDateTime,
    pub last_synced: Option<PrimitiveDateTime>,
    pub cancellation_reason: Option<String>,
    pub browser_info: Option<pii::SecretSerdeValue>,
    pub error_code: Option<String>,
    pub payment_token: Option<String>,
    pub connector_metadata: Option<pii::SecretSerdeValue>,
    pub payment_experience: Option<storage_enums::PaymentExperience>,
    pub payment_method_data: Option<pii::SecretSerdeValue>,
    pub routing_result: Option<serde_json::Value>,
    pub preprocessing_step_id: Option<String>,
    pub error_reason: Option<String>,
    pub multiple_capture_count: Option<i16>,
    pub connector_response_reference_id: Option<String>,
    pub updated_by: String,
    pub authentication_data: Option<pii::SecretSerdeValue>,
    pub encoded_data: Option<Secret<String>>,
    pub merchant_connector_id: Option<id_type::MerchantConnectorAccountId>,
    pub unified_code: Option<String>,
    pub unified_message: Option<String>,
    pub external_three_ds_authentication_attempted: Option<bool>,
    pub authentication_connector: Option<String>,
    pub authentication_id: Option<String>,
    pub fingerprint_id: Option<String>,
    pub charge_id: Option<String>,
    pub client_source: Option<String>,
    pub client_version: Option<String>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub profile_id: id_type::ProfileId,
    pub organization_id: id_type::OrganizationId,
    pub payment_method_type: Option<storage_enums::PaymentMethod>,
    pub payment_method_id: Option<id_type::GlobalPaymentMethodId>,
    pub connector_payment_id: Option<String>,
    pub payment_method_subtype: Option<storage_enums::PaymentMethodType>,
    pub authentication_applied: Option<common_enums::AuthenticationType>,
    pub external_reference_id: Option<String>,
    pub payment_method_billing_address: common_utils::crypto::OptionalEncryptableValue,
    pub id: String,
}

impl PaymentAttempt {
    #[cfg(feature = "v1")]
    pub fn get_payment_method(&self) -> Option<storage_enums::PaymentMethod> {
        self.payment_method
    }

    #[cfg(feature = "v2")]
    pub fn get_payment_method(&self) -> Option<storage_enums::PaymentMethod> {
        self.payment_method_type
    }

    #[cfg(feature = "v1")]
    pub fn get_payment_method_type(&self) -> Option<storage_enums::PaymentMethodType> {
        self.payment_method_type
    }

    #[cfg(feature = "v2")]
    pub fn get_payment_method_type(&self) -> Option<storage_enums::PaymentMethodType> {
        self.payment_method_subtype
    }

    #[cfg(feature = "v1")]
    pub fn get_id(&self) -> &str {
        &self.attempt_id
    }

    #[cfg(feature = "v2")]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    #[cfg(feature = "v1")]
    pub fn get_connector_payment_id(&self) -> Option<&str> {
        self.connector_transaction_id.as_deref()
    }

    #[cfg(feature = "v2")]
    pub fn get_connector_payment_id(&self) -> Option<&str> {
        self.connector_payment_id.as_deref()
    }
}

#[cfg(feature = "v1")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaymentAttempt {
    pub payment_id: id_type::PaymentId,
    pub merchant_id: id_type::MerchantId,
    pub attempt_id: String,
    pub status: storage_enums::AttemptStatus,
    pub amount: MinorUnit,
    pub net_amount: MinorUnit,
    pub currency: Option<storage_enums::Currency>,
    pub save_to_locker: Option<bool>,
    pub connector: Option<String>,
    pub error_message: Option<String>,
    pub offer_amount: Option<MinorUnit>,
    pub surcharge_amount: Option<MinorUnit>,
    pub tax_amount: Option<MinorUnit>,
    pub payment_method_id: Option<String>,
    pub payment_method: Option<storage_enums::PaymentMethod>,
    pub connector_transaction_id: Option<String>,
    pub capture_method: Option<storage_enums::CaptureMethod>,
    #[serde(default, with = "common_utils::custom_serde::iso8601::option")]
    pub capture_on: Option<PrimitiveDateTime>,
    pub confirm: bool,
    pub authentication_type: Option<storage_enums::AuthenticationType>,
    #[serde(with = "common_utils::custom_serde::iso8601")]
    pub created_at: PrimitiveDateTime,
    #[serde(with = "common_utils::custom_serde::iso8601")]
    pub modified_at: PrimitiveDateTime,
    #[serde(default, with = "common_utils::custom_serde::iso8601::option")]
    pub last_synced: Option<PrimitiveDateTime>,
    pub cancellation_reason: Option<String>,
    pub amount_to_capture: Option<MinorUnit>,
    pub mandate_id: Option<String>,
    pub browser_info: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub payment_token: Option<String>,
    pub connector_metadata: Option<serde_json::Value>,
    pub payment_experience: Option<storage_enums::PaymentExperience>,
    pub payment_method_type: Option<storage_enums::PaymentMethodType>,
    pub payment_method_data: Option<serde_json::Value>,
    pub business_sub_label: Option<String>,
    pub straight_through_algorithm: Option<serde_json::Value>,
    pub preprocessing_step_id: Option<String>,
    // providing a location to store mandate details intermediately for transaction
    pub mandate_details: Option<MandateDataType>,
    pub error_reason: Option<String>,
    pub multiple_capture_count: Option<i16>,
    // reference to the payment at connector side
    pub connector_response_reference_id: Option<String>,
    pub amount_capturable: MinorUnit,
    pub updated_by: String,
    pub authentication_data: Option<serde_json::Value>,
    pub encoded_data: Option<String>,
    pub merchant_connector_id: Option<id_type::MerchantConnectorAccountId>,
    pub unified_code: Option<String>,
    pub unified_message: Option<String>,
    pub external_three_ds_authentication_attempted: Option<bool>,
    pub authentication_connector: Option<String>,
    pub authentication_id: Option<String>,
    pub mandate_data: Option<MandateDetails>,
    pub payment_method_billing_address_id: Option<String>,
    pub fingerprint_id: Option<String>,
    pub charge_id: Option<String>,
    pub client_source: Option<String>,
    pub client_version: Option<String>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub profile_id: id_type::ProfileId,
    pub organization_id: id_type::OrganizationId,
    pub shipping_cost: Option<MinorUnit>,
    pub order_tax_amount: Option<MinorUnit>,
}

#[cfg(feature = "v2")]
impl PaymentAttempt {
    pub fn get_total_amount(&self) -> MinorUnit {
        todo!();
    }

    pub fn get_total_surcharge_amount(&self) -> Option<MinorUnit> {
        todo!();
    }
}

#[cfg(feature = "v1")]
impl PaymentAttempt {
    pub fn get_total_amount(&self) -> MinorUnit {
        self.amount
            + self.surcharge_amount.unwrap_or_default()
            + self.tax_amount.unwrap_or_default()
    }

    pub fn get_total_surcharge_amount(&self) -> Option<MinorUnit> {
        self.surcharge_amount
            .map(|surcharge_amount| surcharge_amount + self.tax_amount.unwrap_or_default())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentListFilters {
    pub connector: Vec<String>,
    pub currency: Vec<storage_enums::Currency>,
    pub status: Vec<storage_enums::IntentStatus>,
    pub payment_method: Vec<storage_enums::PaymentMethod>,
    pub payment_method_type: Vec<storage_enums::PaymentMethodType>,
    pub authentication_type: Vec<storage_enums::AuthenticationType>,
}

// #[cfg(feature = "v2")]
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct PaymentAttemptNew {
//     pub payment_id: id_type::PaymentId,
//     pub merchant_id: id_type::MerchantId,
//     pub status: storage_enums::AttemptStatus,
//     pub error_message: Option<String>,
//     pub surcharge_amount: Option<MinorUnit>,
//     pub tax_amount: Option<MinorUnit>,
//     pub payment_method_id: Option<String>,
//     pub confirm: bool,
//     pub authentication_type: Option<storage_enums::AuthenticationType>,
//     pub created_at: PrimitiveDateTime,
//     pub modified_at: PrimitiveDateTime,
//     pub last_synced: Option<PrimitiveDateTime>,
//     pub cancellation_reason: Option<String>,
//     pub browser_info: Option<serde_json::Value>,
//     pub payment_token: Option<String>,
//     pub error_code: Option<String>,
//     pub connector_metadata: Option<serde_json::Value>,
//     pub payment_experience: Option<storage_enums::PaymentExperience>,
//     pub payment_method_data: Option<serde_json::Value>,
//     pub straight_through_algorithm: Option<serde_json::Value>,
//     pub preprocessing_step_id: Option<String>,
//     pub error_reason: Option<String>,
//     pub connector_response_reference_id: Option<String>,
//     pub multiple_capture_count: Option<i16>,
//     pub amount_capturable: MinorUnit,
//     pub updated_by: String,
//     pub merchant_connector_id: Option<id_type::MerchantConnectorAccountId>,
//     pub authentication_data: Option<serde_json::Value>,
//     pub encoded_data: Option<String>,
//     pub unified_code: Option<String>,
//     pub unified_message: Option<String>,
//     pub net_amount: Option<MinorUnit>,
//     pub external_three_ds_authentication_attempted: Option<bool>,
//     pub authentication_connector: Option<String>,
//     pub authentication_id: Option<String>,
//     pub fingerprint_id: Option<String>,
//     pub payment_method_billing_address_id: Option<String>,
//     pub charge_id: Option<String>,
//     pub client_source: Option<String>,
//     pub client_version: Option<String>,
//     pub customer_acceptance: Option<pii::SecretSerdeValue>,
//     pub profile_id: id_type::ProfileId,
//     pub organization_id: id_type::OrganizationId,
//     pub card_network: Option<String>,
// }

#[cfg(feature = "v1")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentAttemptNew {
    pub payment_id: id_type::PaymentId,
    pub merchant_id: id_type::MerchantId,
    pub attempt_id: String,
    pub status: storage_enums::AttemptStatus,
    pub amount: MinorUnit,
    /// amount + surcharge_amount + tax_amount
    /// This field will always be derived before updating in the Database
    pub net_amount: MinorUnit,
    pub currency: Option<storage_enums::Currency>,
    // pub auto_capture: Option<bool>,
    pub save_to_locker: Option<bool>,
    pub connector: Option<String>,
    pub error_message: Option<String>,
    pub offer_amount: Option<MinorUnit>,
    pub surcharge_amount: Option<MinorUnit>,
    pub tax_amount: Option<MinorUnit>,
    pub payment_method_id: Option<String>,
    pub payment_method: Option<storage_enums::PaymentMethod>,
    pub capture_method: Option<storage_enums::CaptureMethod>,
    #[serde(default, with = "common_utils::custom_serde::iso8601::option")]
    pub capture_on: Option<PrimitiveDateTime>,
    pub confirm: bool,
    pub authentication_type: Option<storage_enums::AuthenticationType>,
    #[serde(default, with = "common_utils::custom_serde::iso8601::option")]
    pub created_at: Option<PrimitiveDateTime>,
    #[serde(default, with = "common_utils::custom_serde::iso8601::option")]
    pub modified_at: Option<PrimitiveDateTime>,
    #[serde(default, with = "common_utils::custom_serde::iso8601::option")]
    pub last_synced: Option<PrimitiveDateTime>,
    pub cancellation_reason: Option<String>,
    pub amount_to_capture: Option<MinorUnit>,
    pub mandate_id: Option<String>,
    pub browser_info: Option<serde_json::Value>,
    pub payment_token: Option<String>,
    pub error_code: Option<String>,
    pub connector_metadata: Option<serde_json::Value>,
    pub payment_experience: Option<storage_enums::PaymentExperience>,
    pub payment_method_type: Option<storage_enums::PaymentMethodType>,
    pub payment_method_data: Option<serde_json::Value>,
    pub business_sub_label: Option<String>,
    pub straight_through_algorithm: Option<serde_json::Value>,
    pub preprocessing_step_id: Option<String>,
    pub mandate_details: Option<MandateDataType>,
    pub error_reason: Option<String>,
    pub connector_response_reference_id: Option<String>,
    pub multiple_capture_count: Option<i16>,
    pub amount_capturable: MinorUnit,
    pub updated_by: String,
    pub authentication_data: Option<serde_json::Value>,
    pub encoded_data: Option<String>,
    pub merchant_connector_id: Option<id_type::MerchantConnectorAccountId>,
    pub unified_code: Option<String>,
    pub unified_message: Option<String>,
    pub external_three_ds_authentication_attempted: Option<bool>,
    pub authentication_connector: Option<String>,
    pub authentication_id: Option<String>,
    pub mandate_data: Option<MandateDetails>,
    pub payment_method_billing_address_id: Option<String>,
    pub fingerprint_id: Option<String>,
    pub charge_id: Option<String>,
    pub client_source: Option<String>,
    pub client_version: Option<String>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub profile_id: id_type::ProfileId,
    pub organization_id: id_type::OrganizationId,
    pub shipping_cost: Option<MinorUnit>,
    pub order_tax_amount: Option<MinorUnit>,
}

#[cfg(feature = "v1")]
impl PaymentAttemptNew {
    /// returns amount + surcharge_amount + tax_amount
    pub fn calculate_net_amount(&self) -> MinorUnit {
        self.amount
            + self.surcharge_amount.unwrap_or_default()
            + self.tax_amount.unwrap_or_default()
            + self.shipping_cost.unwrap_or_default()
            + self.order_tax_amount.unwrap_or_default()
    }

    pub fn populate_derived_fields(self) -> Self {
        let mut payment_attempt_new = self;
        payment_attempt_new.net_amount = payment_attempt_new.calculate_net_amount();
        payment_attempt_new
    }
}

#[cfg(feature = "v1")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentAttemptUpdate {
    Update {
        amount: MinorUnit,
        currency: storage_enums::Currency,
        status: storage_enums::AttemptStatus,
        authentication_type: Option<storage_enums::AuthenticationType>,
        payment_method: Option<storage_enums::PaymentMethod>,
        payment_token: Option<String>,
        payment_method_data: Option<serde_json::Value>,
        payment_method_type: Option<storage_enums::PaymentMethodType>,
        payment_experience: Option<storage_enums::PaymentExperience>,
        business_sub_label: Option<String>,
        amount_to_capture: Option<MinorUnit>,
        capture_method: Option<storage_enums::CaptureMethod>,
        surcharge_amount: Option<MinorUnit>,
        tax_amount: Option<MinorUnit>,
        fingerprint_id: Option<String>,
        payment_method_billing_address_id: Option<String>,
        updated_by: String,
    },
    UpdateTrackers {
        payment_token: Option<String>,
        connector: Option<String>,
        straight_through_algorithm: Option<serde_json::Value>,
        amount_capturable: Option<MinorUnit>,
        surcharge_amount: Option<MinorUnit>,
        tax_amount: Option<MinorUnit>,
        updated_by: String,
        merchant_connector_id: Option<id_type::MerchantConnectorAccountId>,
    },
    AuthenticationTypeUpdate {
        authentication_type: storage_enums::AuthenticationType,
        updated_by: String,
    },
    ConfirmUpdate {
        amount: MinorUnit,
        currency: storage_enums::Currency,
        status: storage_enums::AttemptStatus,
        authentication_type: Option<storage_enums::AuthenticationType>,
        capture_method: Option<storage_enums::CaptureMethod>,
        payment_method: Option<storage_enums::PaymentMethod>,
        browser_info: Option<serde_json::Value>,
        connector: Option<String>,
        payment_token: Option<String>,
        payment_method_data: Option<serde_json::Value>,
        payment_method_type: Option<storage_enums::PaymentMethodType>,
        payment_experience: Option<storage_enums::PaymentExperience>,
        business_sub_label: Option<String>,
        straight_through_algorithm: Option<serde_json::Value>,
        error_code: Option<Option<String>>,
        error_message: Option<Option<String>>,
        amount_capturable: Option<MinorUnit>,
        updated_by: String,
        surcharge_amount: Option<MinorUnit>,
        tax_amount: Option<MinorUnit>,
        merchant_connector_id: Option<id_type::MerchantConnectorAccountId>,
        external_three_ds_authentication_attempted: Option<bool>,
        authentication_connector: Option<String>,
        authentication_id: Option<String>,
        payment_method_billing_address_id: Option<String>,
        fingerprint_id: Option<String>,
        payment_method_id: Option<String>,
        client_source: Option<String>,
        client_version: Option<String>,
        customer_acceptance: Option<pii::SecretSerdeValue>,
        shipping_cost: Option<MinorUnit>,
        order_tax_amount: Option<MinorUnit>,
    },
    RejectUpdate {
        status: storage_enums::AttemptStatus,
        error_code: Option<Option<String>>,
        error_message: Option<Option<String>>,
        updated_by: String,
    },
    BlocklistUpdate {
        status: storage_enums::AttemptStatus,
        error_code: Option<Option<String>>,
        error_message: Option<Option<String>>,
        updated_by: String,
    },
    PaymentMethodDetailsUpdate {
        payment_method_id: Option<String>,
        updated_by: String,
    },
    VoidUpdate {
        status: storage_enums::AttemptStatus,
        cancellation_reason: Option<String>,
        updated_by: String,
    },
    ResponseUpdate {
        status: storage_enums::AttemptStatus,
        connector: Option<String>,
        connector_transaction_id: Option<String>,
        authentication_type: Option<storage_enums::AuthenticationType>,
        payment_method_id: Option<String>,
        mandate_id: Option<String>,
        connector_metadata: Option<serde_json::Value>,
        payment_token: Option<String>,
        error_code: Option<Option<String>>,
        error_message: Option<Option<String>>,
        error_reason: Option<Option<String>>,
        connector_response_reference_id: Option<String>,
        amount_capturable: Option<MinorUnit>,
        updated_by: String,
        authentication_data: Option<serde_json::Value>,
        encoded_data: Option<String>,
        unified_code: Option<Option<String>>,
        unified_message: Option<Option<String>>,
        payment_method_data: Option<serde_json::Value>,
        charge_id: Option<String>,
    },
    UnresolvedResponseUpdate {
        status: storage_enums::AttemptStatus,
        connector: Option<String>,
        connector_transaction_id: Option<String>,
        payment_method_id: Option<String>,
        error_code: Option<Option<String>>,
        error_message: Option<Option<String>>,
        error_reason: Option<Option<String>>,
        connector_response_reference_id: Option<String>,
        updated_by: String,
    },
    StatusUpdate {
        status: storage_enums::AttemptStatus,
        updated_by: String,
    },
    ErrorUpdate {
        connector: Option<String>,
        status: storage_enums::AttemptStatus,
        error_code: Option<Option<String>>,
        error_message: Option<Option<String>>,
        error_reason: Option<Option<String>>,
        amount_capturable: Option<MinorUnit>,
        updated_by: String,
        unified_code: Option<Option<String>>,
        unified_message: Option<Option<String>>,
        connector_transaction_id: Option<String>,
        payment_method_data: Option<serde_json::Value>,
        authentication_type: Option<storage_enums::AuthenticationType>,
    },
    CaptureUpdate {
        amount_to_capture: Option<MinorUnit>,
        multiple_capture_count: Option<i16>,
        updated_by: String,
    },
    AmountToCaptureUpdate {
        status: storage_enums::AttemptStatus,
        amount_capturable: MinorUnit,
        updated_by: String,
    },
    PreprocessingUpdate {
        status: storage_enums::AttemptStatus,
        payment_method_id: Option<String>,
        connector_metadata: Option<serde_json::Value>,
        preprocessing_step_id: Option<String>,
        connector_transaction_id: Option<String>,
        connector_response_reference_id: Option<String>,
        updated_by: String,
    },
    ConnectorResponse {
        authentication_data: Option<serde_json::Value>,
        encoded_data: Option<String>,
        connector_transaction_id: Option<String>,
        connector: Option<String>,
        charge_id: Option<String>,
        updated_by: String,
    },
    IncrementalAuthorizationAmountUpdate {
        amount: MinorUnit,
        amount_capturable: MinorUnit,
    },
    AuthenticationUpdate {
        status: storage_enums::AttemptStatus,
        external_three_ds_authentication_attempted: Option<bool>,
        authentication_connector: Option<String>,
        authentication_id: Option<String>,
        updated_by: String,
    },
    ManualUpdate {
        status: Option<storage_enums::AttemptStatus>,
        error_code: Option<String>,
        error_message: Option<String>,
        error_reason: Option<String>,
        updated_by: String,
        unified_code: Option<String>,
        unified_message: Option<String>,
        connector_transaction_id: Option<String>,
    },
}

// TODO: Add fields as necessary
#[cfg(feature = "v2")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentAttemptUpdate {}

#[cfg(feature = "v2")]
impl ForeignIDRef for PaymentAttempt {
    fn foreign_id(&self) -> String {
        todo!()
    }
}

#[cfg(feature = "v1")]
impl ForeignIDRef for PaymentAttempt {
    fn foreign_id(&self) -> String {
        self.attempt_id.clone()
    }
}

#[cfg(feature = "v1")]
#[async_trait::async_trait]
impl behaviour::Conversion for PaymentAttempt {
    type DstType = DieselPaymentAttempt;
    type NewDstType = DieselPaymentAttemptNew;

    async fn convert(self) -> CustomResult<Self::DstType, ValidationError> {
        let card_network = self
            .payment_method_data
            .as_ref()
            .and_then(|data| data.as_object())
            .and_then(|card| card.get("card"))
            .and_then(|data| data.as_object())
            .and_then(|card| card.get("card_network"))
            .and_then(|network| network.as_str())
            .map(|network| network.to_string());
        Ok(DieselPaymentAttempt {
            payment_id: self.payment_id,
            merchant_id: self.merchant_id,
            attempt_id: self.attempt_id,
            status: self.status,
            amount: self.amount,
            currency: self.currency,
            save_to_locker: self.save_to_locker,
            connector: self.connector,
            error_message: self.error_message,
            offer_amount: self.offer_amount,
            surcharge_amount: self.surcharge_amount,
            tax_amount: self.tax_amount,
            payment_method_id: self.payment_method_id,
            payment_method: self.payment_method,
            connector_transaction_id: self.connector_transaction_id,
            capture_method: self.capture_method,
            capture_on: self.capture_on,
            confirm: self.confirm,
            authentication_type: self.authentication_type,
            created_at: self.created_at,
            modified_at: self.modified_at,
            last_synced: self.last_synced,
            cancellation_reason: self.cancellation_reason,
            amount_to_capture: self.amount_to_capture,
            mandate_id: self.mandate_id,
            browser_info: self.browser_info,
            error_code: self.error_code,
            payment_token: self.payment_token,
            connector_metadata: self.connector_metadata,
            payment_experience: self.payment_experience,
            payment_method_type: self.payment_method_type,
            payment_method_data: self.payment_method_data,
            business_sub_label: self.business_sub_label,
            straight_through_algorithm: self.straight_through_algorithm,
            preprocessing_step_id: self.preprocessing_step_id,
            mandate_details: self.mandate_details.map(Into::into),
            error_reason: self.error_reason,
            multiple_capture_count: self.multiple_capture_count,
            connector_response_reference_id: self.connector_response_reference_id,
            amount_capturable: self.amount_capturable,
            updated_by: self.updated_by,
            merchant_connector_id: self.merchant_connector_id,
            authentication_data: self.authentication_data,
            encoded_data: self.encoded_data,
            unified_code: self.unified_code,
            unified_message: self.unified_message,
            net_amount: Some(self.net_amount),
            external_three_ds_authentication_attempted: self
                .external_three_ds_authentication_attempted,
            authentication_connector: self.authentication_connector,
            authentication_id: self.authentication_id,
            mandate_data: self.mandate_data.map(Into::into),
            fingerprint_id: self.fingerprint_id,
            payment_method_billing_address_id: self.payment_method_billing_address_id,
            charge_id: self.charge_id,
            client_source: self.client_source,
            client_version: self.client_version,
            customer_acceptance: self.customer_acceptance,
            profile_id: self.profile_id,
            organization_id: self.organization_id,
            card_network,
            order_tax_amount: self.order_tax_amount,
            shipping_cost: self.shipping_cost,
        })
    }

    async fn convert_back(
        _state: &KeyManagerState,
        storage_model: Self::DstType,
        _key: &Secret<Vec<u8>>,
        _key_manager_identifier: keymanager::Identifier,
    ) -> CustomResult<Self, ValidationError>
    where
        Self: Sized,
    {
        async {
            let net_amount = storage_model.get_or_calculate_net_amount();
            Ok::<Self, error_stack::Report<common_utils::errors::CryptoError>>(Self {
                payment_id: storage_model.payment_id,
                merchant_id: storage_model.merchant_id,
                attempt_id: storage_model.attempt_id,
                status: storage_model.status,
                amount: storage_model.amount,
                net_amount,
                currency: storage_model.currency,
                save_to_locker: storage_model.save_to_locker,
                connector: storage_model.connector,
                error_message: storage_model.error_message,
                offer_amount: storage_model.offer_amount,
                surcharge_amount: storage_model.surcharge_amount,
                tax_amount: storage_model.tax_amount,
                payment_method_id: storage_model.payment_method_id,
                payment_method: storage_model.payment_method,
                connector_transaction_id: storage_model.connector_transaction_id,
                capture_method: storage_model.capture_method,
                capture_on: storage_model.capture_on,
                confirm: storage_model.confirm,
                authentication_type: storage_model.authentication_type,
                created_at: storage_model.created_at,
                modified_at: storage_model.modified_at,
                last_synced: storage_model.last_synced,
                cancellation_reason: storage_model.cancellation_reason,
                amount_to_capture: storage_model.amount_to_capture,
                mandate_id: storage_model.mandate_id,
                browser_info: storage_model.browser_info,
                error_code: storage_model.error_code,
                payment_token: storage_model.payment_token,
                connector_metadata: storage_model.connector_metadata,
                payment_experience: storage_model.payment_experience,
                payment_method_type: storage_model.payment_method_type,
                payment_method_data: storage_model.payment_method_data,
                business_sub_label: storage_model.business_sub_label,
                straight_through_algorithm: storage_model.straight_through_algorithm,
                preprocessing_step_id: storage_model.preprocessing_step_id,
                mandate_details: storage_model.mandate_details.map(Into::into),
                error_reason: storage_model.error_reason,
                multiple_capture_count: storage_model.multiple_capture_count,
                connector_response_reference_id: storage_model.connector_response_reference_id,
                amount_capturable: storage_model.amount_capturable,
                updated_by: storage_model.updated_by,
                authentication_data: storage_model.authentication_data,
                encoded_data: storage_model.encoded_data,
                merchant_connector_id: storage_model.merchant_connector_id,
                unified_code: storage_model.unified_code,
                unified_message: storage_model.unified_message,
                external_three_ds_authentication_attempted: storage_model
                    .external_three_ds_authentication_attempted,
                authentication_connector: storage_model.authentication_connector,
                authentication_id: storage_model.authentication_id,
                mandate_data: storage_model.mandate_data.map(Into::into),
                payment_method_billing_address_id: storage_model.payment_method_billing_address_id,
                fingerprint_id: storage_model.fingerprint_id,
                charge_id: storage_model.charge_id,
                client_source: storage_model.client_source,
                client_version: storage_model.client_version,
                customer_acceptance: storage_model.customer_acceptance,
                profile_id: storage_model.profile_id,
                organization_id: storage_model.organization_id,
                order_tax_amount: storage_model.order_tax_amount,
                shipping_cost: storage_model.shipping_cost,
            })
        }
        .await
        .change_context(ValidationError::InvalidValue {
            message: "Failed while decrypting payment attempt".to_string(),
        })
    }

    async fn construct_new(self) -> CustomResult<Self::NewDstType, ValidationError> {
        let card_network = self
            .payment_method_data
            .as_ref()
            .and_then(|data| data.as_object())
            .and_then(|card| card.get("card"))
            .and_then(|data| data.as_object())
            .and_then(|card| card.get("card_network"))
            .and_then(|network| network.as_str())
            .map(|network| network.to_string());
        Ok(DieselPaymentAttemptNew {
            payment_id: self.payment_id,
            merchant_id: self.merchant_id,
            attempt_id: self.attempt_id,
            status: self.status,
            amount: self.amount,
            currency: self.currency,
            save_to_locker: self.save_to_locker,
            connector: self.connector,
            error_message: self.error_message,
            offer_amount: self.offer_amount,
            surcharge_amount: self.surcharge_amount,
            tax_amount: self.tax_amount,
            payment_method_id: self.payment_method_id,
            payment_method: self.payment_method,
            capture_method: self.capture_method,
            capture_on: self.capture_on,
            confirm: self.confirm,
            authentication_type: self.authentication_type,
            created_at: self.created_at,
            modified_at: self.modified_at,
            last_synced: self.last_synced,
            cancellation_reason: self.cancellation_reason,
            amount_to_capture: self.amount_to_capture,
            mandate_id: self.mandate_id,
            browser_info: self.browser_info,
            payment_token: self.payment_token,
            error_code: self.error_code,
            connector_metadata: self.connector_metadata,
            payment_experience: self.payment_experience,
            payment_method_type: self.payment_method_type,
            payment_method_data: self.payment_method_data,
            business_sub_label: self.business_sub_label,
            straight_through_algorithm: self.straight_through_algorithm,
            preprocessing_step_id: self.preprocessing_step_id,
            mandate_details: self.mandate_details.map(Into::into),
            error_reason: self.error_reason,
            connector_response_reference_id: self.connector_response_reference_id,
            multiple_capture_count: self.multiple_capture_count,
            amount_capturable: self.amount_capturable,
            updated_by: self.updated_by,
            merchant_connector_id: self.merchant_connector_id,
            authentication_data: self.authentication_data,
            encoded_data: self.encoded_data,
            unified_code: self.unified_code,
            unified_message: self.unified_message,
            net_amount: Some(self.net_amount),
            external_three_ds_authentication_attempted: self
                .external_three_ds_authentication_attempted,
            authentication_connector: self.authentication_connector,
            authentication_id: self.authentication_id,
            mandate_data: self.mandate_data.map(Into::into),
            fingerprint_id: self.fingerprint_id,
            payment_method_billing_address_id: self.payment_method_billing_address_id,
            charge_id: self.charge_id,
            client_source: self.client_source,
            client_version: self.client_version,
            customer_acceptance: self.customer_acceptance,
            profile_id: self.profile_id,
            organization_id: self.organization_id,
            card_network,
            order_tax_amount: self.order_tax_amount,
            shipping_cost: self.shipping_cost,
        })
    }
}

#[cfg(feature = "v2")]
#[async_trait::async_trait]
impl behaviour::Conversion for PaymentAttempt {
    type DstType = DieselPaymentAttempt;
    type NewDstType = DieselPaymentAttemptNew;

    async fn convert(self) -> CustomResult<Self::DstType, ValidationError> {
        use common_utils::encryption::Encryption;

        let card_network = self
            .payment_method_data
            .as_ref()
            .and_then(|data| data.peek().as_object())
            .and_then(|card| card.get("card"))
            .and_then(|data| data.as_object())
            .and_then(|card| card.get("card_network"))
            .and_then(|network| network.as_str())
            .map(|network| network.to_string());

        let Self {
            payment_id,
            merchant_id,
            status,
            error_message,
            amount_details,
            authentication_type,
            created_at,
            modified_at,
            last_synced,
            cancellation_reason,
            browser_info,
            error_code,
            payment_token,
            connector_metadata,
            payment_experience,
            payment_method_data,
            routing_result,
            preprocessing_step_id,
            error_reason,
            multiple_capture_count,
            connector_response_reference_id,
            updated_by,
            authentication_data,
            encoded_data,
            merchant_connector_id,
            unified_code,
            unified_message,
            external_three_ds_authentication_attempted,
            authentication_connector,
            authentication_id,
            fingerprint_id,
            charge_id,
            client_source,
            client_version,
            customer_acceptance,
            profile_id,
            organization_id,
            payment_method_type,
            connector_payment_id,
            payment_method_subtype,
            authentication_applied,
            external_reference_id,
            id,
            payment_method_id,
            payment_method_billing_address,
            connector,
        } = self;

        let AmountDetails {
            net_amount,
            tax_on_surcharge,
            surcharge_amount,
            order_tax_amount,
            shipping_cost,
            amount_capturable,
            amount_to_capture,
        } = amount_details;

        Ok(DieselPaymentAttempt {
            payment_id,
            merchant_id,
            id,
            status,
            error_message,
            payment_method_id,
            payment_method_type_v2: payment_method_type,
            connector_payment_id,
            authentication_type,
            created_at,
            modified_at,
            last_synced,
            cancellation_reason,
            amount_to_capture,
            browser_info,
            error_code,
            payment_token,
            connector_metadata,
            payment_experience,
            payment_method_subtype,
            payment_method_data,
            preprocessing_step_id,
            error_reason,
            multiple_capture_count,
            connector_response_reference_id,
            amount_capturable,
            updated_by,
            merchant_connector_id,
            authentication_data,
            encoded_data,
            unified_code,
            unified_message,
            net_amount,
            external_three_ds_authentication_attempted,
            authentication_connector,
            authentication_id,
            fingerprint_id,
            charge_id,
            client_source,
            client_version,
            customer_acceptance,
            profile_id,
            organization_id,
            card_network,
            order_tax_amount,
            shipping_cost,
            routing_result,
            authentication_applied,
            external_reference_id,
            connector,
            surcharge_amount,
            tax_on_surcharge,
            payment_method_billing_address: payment_method_billing_address.map(Encryption::from),
        })
    }

    async fn convert_back(
        state: &KeyManagerState,
        storage_model: Self::DstType,
        key: &Secret<Vec<u8>>,
        key_manager_identifier: keymanager::Identifier,
    ) -> CustomResult<Self, ValidationError>
    where
        Self: Sized,
    {
        async {
            let amount_details = AmountDetails {
                net_amount: storage_model.net_amount,
                tax_on_surcharge: storage_model.tax_on_surcharge,
                surcharge_amount: storage_model.surcharge_amount,
                order_tax_amount: storage_model.order_tax_amount,
                shipping_cost: storage_model.shipping_cost,
                amount_capturable: storage_model.amount_capturable,
                amount_to_capture: storage_model.amount_to_capture,
            };

            let inner_decrypt = |inner| async {
                crate::type_encryption::crypto_operation(
                    state,
                    common_utils::type_name!(Self::DstType),
                    crate::type_encryption::CryptoOperation::DecryptOptional(inner),
                    key_manager_identifier.clone(),
                    key.peek(),
                )
                .await
                .and_then(|val| val.try_into_optionaloperation())
            };

            Ok::<Self, error_stack::Report<common_utils::errors::CryptoError>>(Self {
                payment_id: storage_model.payment_id,
                merchant_id: storage_model.merchant_id,
                id: storage_model.id,
                status: storage_model.status,
                amount_details,
                error_message: storage_model.error_message,
                payment_method_id: storage_model.payment_method_id,
                payment_method_type: storage_model.payment_method_type_v2,
                connector_payment_id: storage_model.connector_payment_id,
                authentication_type: storage_model.authentication_type,
                created_at: storage_model.created_at,
                modified_at: storage_model.modified_at,
                last_synced: storage_model.last_synced,
                cancellation_reason: storage_model.cancellation_reason,
                browser_info: storage_model.browser_info,
                error_code: storage_model.error_code,
                payment_token: storage_model.payment_token,
                connector_metadata: storage_model.connector_metadata,
                payment_experience: storage_model.payment_experience,
                payment_method_data: storage_model.payment_method_data,
                routing_result: storage_model.routing_result,
                preprocessing_step_id: storage_model.preprocessing_step_id,
                error_reason: storage_model.error_reason,
                multiple_capture_count: storage_model.multiple_capture_count,
                connector_response_reference_id: storage_model.connector_response_reference_id,
                updated_by: storage_model.updated_by,
                authentication_data: storage_model.authentication_data,
                encoded_data: storage_model.encoded_data,
                merchant_connector_id: storage_model.merchant_connector_id,
                unified_code: storage_model.unified_code,
                unified_message: storage_model.unified_message,
                external_three_ds_authentication_attempted: storage_model
                    .external_three_ds_authentication_attempted,
                authentication_connector: storage_model.authentication_connector,
                authentication_id: storage_model.authentication_id,
                fingerprint_id: storage_model.fingerprint_id,
                charge_id: storage_model.charge_id,
                client_source: storage_model.client_source,
                client_version: storage_model.client_version,
                customer_acceptance: storage_model.customer_acceptance,
                profile_id: storage_model.profile_id,
                organization_id: storage_model.organization_id,
                payment_method_subtype: storage_model.payment_method_subtype,
                authentication_applied: storage_model.authentication_applied,
                external_reference_id: storage_model.external_reference_id,
                connector: storage_model.connector,
                payment_method_billing_address: inner_decrypt(
                    storage_model.payment_method_billing_address,
                )
                .await?,
            })
        }
        .await
        .change_context(ValidationError::InvalidValue {
            message: "Failed while decrypting payment attempt".to_string(),
        })
    }

    async fn construct_new(self) -> CustomResult<Self::NewDstType, ValidationError> {
        use common_utils::encryption::Encryption;

        let card_network = self
            .payment_method_data
            .as_ref()
            .and_then(|data| data.peek().as_object())
            .and_then(|card| card.get("card"))
            .and_then(|data| data.as_object())
            .and_then(|card| card.get("card_network"))
            .and_then(|network| network.as_str())
            .map(|network| network.to_string());

        Ok(DieselPaymentAttemptNew {
            payment_id: self.payment_id,
            merchant_id: self.merchant_id,
            status: self.status,
            error_message: self.error_message,
            surcharge_amount: self.amount_details.surcharge_amount,
            tax_on_surcharge: self.amount_details.tax_on_surcharge,
            payment_method_id: self.payment_method_id,
            authentication_type: self.authentication_type,
            created_at: self.created_at,
            modified_at: self.modified_at,
            last_synced: self.last_synced,
            cancellation_reason: self.cancellation_reason,
            browser_info: self.browser_info,
            payment_token: self.payment_token,
            error_code: self.error_code,
            connector_metadata: self.connector_metadata,
            payment_experience: self.payment_experience,
            payment_method_data: self.payment_method_data,
            preprocessing_step_id: self.preprocessing_step_id,
            error_reason: self.error_reason,
            connector_response_reference_id: self.connector_response_reference_id,
            multiple_capture_count: self.multiple_capture_count,
            amount_capturable: self.amount_details.amount_capturable,
            updated_by: self.updated_by,
            merchant_connector_id: self.merchant_connector_id,
            authentication_data: self.authentication_data,
            encoded_data: self.encoded_data,
            unified_code: self.unified_code,
            unified_message: self.unified_message,
            net_amount: self.amount_details.net_amount,
            external_three_ds_authentication_attempted: self
                .external_three_ds_authentication_attempted,
            authentication_connector: self.authentication_connector,
            authentication_id: self.authentication_id,
            fingerprint_id: self.fingerprint_id,
            charge_id: self.charge_id,
            client_source: self.client_source,
            client_version: self.client_version,
            customer_acceptance: self.customer_acceptance,
            profile_id: self.profile_id,
            organization_id: self.organization_id,
            card_network,
            order_tax_amount: self.amount_details.order_tax_amount,
            shipping_cost: self.amount_details.shipping_cost,
            amount_to_capture: self.amount_details.amount_to_capture,
            payment_method_billing_address: self
                .payment_method_billing_address
                .map(Encryption::from),
        })
    }
}

#[cfg(feature = "v2")]
impl From<PaymentAttemptUpdate> for diesel_models::PaymentAttemptUpdateInternal {
    fn from(update: PaymentAttemptUpdate) -> Self {
        todo!()
    }
}
