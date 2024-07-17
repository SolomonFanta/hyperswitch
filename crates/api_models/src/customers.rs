use common_utils::{crypto, custom_serde, id_type, pii};
use masking::Secret;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::payments;

/// The customer details
#[cfg(not(feature = "v2"))]
#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct CustomerRequest {
    /// The identifier for the customer object. If not provided the customer ID will be autogenerated.
    #[schema(value_type = Option<String>, max_length = 64, min_length = 1, example = "cus_y3oqhf46pyzuxjbcn2giaqnb44")]
    pub customer_id: Option<id_type::CustomerId>,
    /// The identifier for the Merchant Account
    #[schema(max_length = 255, example = "y3oqhf46pyzuxjbcn2giaqnb44")]
    #[serde(default = "unknown_merchant", skip)]
    pub merchant_id: String,
    /// The customer's name
    #[schema(max_length = 255, value_type = Option<String>, example = "Jon Test")]
    pub name: Option<Secret<String>>,
    /// The customer's email address
    #[schema(value_type = Option<String>, max_length = 255, example = "JonTest@test.com")]
    pub email: Option<pii::Email>,
    /// The customer's phone number
    #[schema(value_type = Option<String>, max_length = 255, example = "9123456789")]
    pub phone: Option<Secret<String>>,
    /// An arbitrary string that you can attach to a customer object.
    #[schema(max_length = 255, example = "First Customer")]
    pub description: Option<String>,
    /// The country code for the customer phone number
    #[schema(max_length = 255, example = "+65")]
    pub phone_country_code: Option<String>,
    /// The address for the customer
    #[schema(value_type = Option<AddressDetails>)]
    pub address: Option<payments::AddressDetails>,
    /// You can specify up to 50 keys, with key names up to 40 characters long and values up to 500
    /// characters long. Metadata is useful for storing additional, structured information on an
    /// object.
    #[schema(value_type = Option<Object>,example = json!({ "city": "NY", "unit": "245" }))]
    pub metadata: Option<pii::SecretSerdeValue>,
}

#[cfg(not(feature = "v2"))]
impl CustomerRequest {
    pub fn get_merchant_reference_id(&self) -> Option<id_type::CustomerId> {
        Some(
            self.customer_id
                .to_owned()
                .unwrap_or_else(common_utils::generate_customer_id_of_default_length),
        )
    }
    pub fn get_address(&self) -> Option<payments::AddressDetails> {
        self.address.clone()
    }
    pub fn get_optional_email(&self) -> Option<pii::Email> {
        self.email.clone()
    }
}

/// The customer details
#[cfg(feature = "v2")]
#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct CustomerRequest {
    /// The merchant identifier for the customer object.
    #[schema(value_type = Option<String>, max_length = 64, min_length = 1, example = "cus_y3oqhf46pyzuxjbcn2giaqnb44")]
    pub merchant_reference_id: Option<id_type::CustomerId>,
    /// The customer's name
    #[schema(max_length = 255, value_type = String, example = "Jon Test")]
    pub name: Secret<String>,
    /// The customer's email address
    #[schema(value_type = String, max_length = 255, example = "JonTest@test.com")]
    pub email: pii::Email,
    /// The customer's phone number
    #[schema(value_type = Option<String>, max_length = 255, example = "9123456789")]
    pub phone: Option<Secret<String>>,
    /// An arbitrary string that you can attach to a customer object.
    #[schema(max_length = 255, example = "First Customer")]
    pub description: Option<String>,
    /// The country code for the customer phone number
    #[schema(max_length = 255, example = "+65")]
    pub phone_country_code: Option<String>,
    /// The default billing address for the customer
    #[schema(value_type = Option<AddressDetails>)]
    pub default_billing_address: Option<payments::AddressDetails>,
    /// The default shipping address for the customer
    #[schema(value_type = Option<AddressDetails>)]
    pub default_shipping_address: Option<payments::AddressDetails>,
    /// You can specify up to 50 keys, with key names up to 40 characters long and values up to 500
    /// characters long. Metadata is useful for storing additional, structured information on an
    /// object.
    #[schema(value_type = Option<Object>,example = json!({ "city": "NY", "unit": "245" }))]
    pub metadata: Option<pii::SecretSerdeValue>,
}

#[cfg(feature = "v2")]
impl CustomerRequest {
    pub fn get_merchant_reference_id(&self) -> Option<id_type::CustomerId> {
        self.merchant_reference_id.clone()
    }

    pub fn get_default_customer_billing_address(&self) -> Option<payments::AddressDetails> {
        self.default_billing_address.clone()
    }

    pub fn get_default_customer_shipping_address(&self) -> Option<payments::AddressDetails> {
        self.default_shipping_address.clone()
    }

    pub fn get_optional_email(&self) -> Option<pii::Email> {
        Some(self.email.clone())
    }
}

#[cfg(not(feature = "v2"))]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CustomerResponse {
    /// The identifier for the customer object
    #[schema(value_type = String, max_length = 64, min_length = 1, example = "cus_y3oqhf46pyzuxjbcn2giaqnb44")]
    pub customer_id: id_type::CustomerId,
    /// The customer's name
    #[schema(max_length = 255, value_type = Option<String>, example = "Jon Test")]
    pub name: crypto::OptionalEncryptableName,
    /// The customer's email address
    #[schema(value_type = Option<String>,max_length = 255, example = "JonTest@test.com")]
    pub email: crypto::OptionalEncryptableEmail,
    /// The customer's phone number
    #[schema(value_type = Option<String>,max_length = 255, example = "9123456789")]
    pub phone: crypto::OptionalEncryptablePhone,
    /// The country code for the customer phone number
    #[schema(max_length = 255, example = "+65")]
    pub phone_country_code: Option<String>,
    /// An arbitrary string that you can attach to a customer object.
    #[schema(max_length = 255, example = "First Customer")]
    pub description: Option<String>,
    /// The address for the customer
    #[schema(value_type = Option<AddressDetails>)]
    pub address: Option<payments::AddressDetails>,
    ///  A timestamp (ISO 8601 code) that determines when the customer was created
    #[schema(value_type = PrimitiveDateTime,example = "2023-01-18T11:04:09.922Z")]
    #[serde(with = "custom_serde::iso8601")]
    pub created_at: time::PrimitiveDateTime,
    /// You can specify up to 50 keys, with key names up to 40 characters long and values up to 500
    /// characters long. Metadata is useful for storing additional, structured information on an
    /// object.
    #[schema(value_type = Option<Object>,example = json!({ "city": "NY", "unit": "245" }))]
    pub metadata: Option<pii::SecretSerdeValue>,
    /// The identifier for the default payment method.
    #[schema(max_length = 64, example = "pm_djh2837dwduh890123")]
    pub default_payment_method_id: Option<String>,
}

#[cfg(not(feature = "v2"))]
impl CustomerResponse {
    pub fn get_merchant_reference_id(&self) -> Option<id_type::CustomerId> {
        Some(self.customer_id.clone())
    }
}

#[cfg(feature = "v2")]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CustomerResponse {
    /// The identifier for the customer object
    #[schema(value_type = String, max_length = 64, min_length = 1, example = "cus_y3oqhf46pyzuxjbcn2giaqnb44")]
    pub merchant_reference_id: Option<id_type::CustomerId>,
    /// The customer's name
    #[schema(max_length = 255, value_type = Option<String>, example = "Jon Test")]
    pub name: crypto::OptionalEncryptableName,
    /// The customer's email address
    #[schema(value_type = Option<String> ,max_length = 255, example = "JonTest@test.com")]
    pub email: crypto::OptionalEncryptableEmail,
    /// The customer's phone number
    #[schema(value_type = Option<String>,max_length = 255, example = "9123456789")]
    pub phone: crypto::OptionalEncryptablePhone,
    /// The country code for the customer phone number
    #[schema(max_length = 255, example = "+65")]
    pub phone_country_code: Option<String>,
    /// An arbitrary string that you can attach to a customer object.
    #[schema(max_length = 255, example = "First Customer")]
    pub description: Option<String>,
    /// The default billing address for the customer
    #[schema(value_type = Option<AddressDetails>)]
    pub default_billing_address: Option<payments::AddressDetails>,
    /// The default shipping address for the customer
    #[schema(value_type = Option<AddressDetails>)]
    pub default_shipping_address: Option<payments::AddressDetails>,
    ///  A timestamp (ISO 8601 code) that determines when the customer was created
    #[schema(value_type = PrimitiveDateTime,example = "2023-01-18T11:04:09.922Z")]
    #[serde(with = "custom_serde::iso8601")]
    pub created_at: time::PrimitiveDateTime,
    /// You can specify up to 50 keys, with key names up to 40 characters long and values up to 500
    /// characters long. Metadata is useful for storing additional, structured information on an
    /// object.
    #[schema(value_type = Option<Object>,example = json!({ "city": "NY", "unit": "245" }))]
    pub metadata: Option<pii::SecretSerdeValue>,
    /// The identifier for the default payment method.
    #[schema(max_length = 64, example = "pm_djh2837dwduh890123")]
    pub default_payment_method_id: Option<String>,
}

#[cfg(feature = "v2")]
impl CustomerResponse {
    pub fn get_merchant_reference_id(&self) -> Option<id_type::CustomerId> {
        self.merchant_reference_id.clone()
    }
}

#[cfg(all(not(feature = "v2")))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomerId {
    pub customer_id: id_type::CustomerId,
}

#[cfg(all(not(feature = "v2")))]
impl CustomerId {
    pub fn get_merchant_reference_id(&self) -> id_type::CustomerId {
        self.customer_id.clone()
    }

    pub fn new_customer_id_struct(cust: id_type::CustomerId) -> CustomerId {
        CustomerId { customer_id: cust }
    }
}

#[cfg(feature = "v2")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomerId {
    pub merchant_reference_id: id_type::CustomerId,
}

#[cfg(all(feature = "v2"))]
impl CustomerId {
    pub fn get_merchant_reference_id(&self) -> id_type::CustomerId {
        self.merchant_reference_id.clone()
    }

    pub fn new_customer_id_struct(cust: id_type::CustomerId) -> CustomerId {
        CustomerId {
            merchant_reference_id: cust,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CustomerDeleteResponse {
    /// The identifier for the customer object
    #[schema(value_type = String, max_length = 255, example = "cus_y3oqhf46pyzuxjbcn2giaqnb44")]
    pub customer_id: id_type::CustomerId,
    /// Whether customer was deleted or not
    #[schema(example = false)]
    pub customer_deleted: bool,
    /// Whether address was deleted or not
    #[schema(example = false)]
    pub address_deleted: bool,
    /// Whether payment methods deleted or not
    #[schema(example = false)]
    pub payment_methods_deleted: bool,
}

#[cfg(not(feature = "v2"))]
fn unknown_merchant() -> String {
    String::from("merchant_unknown")
}
