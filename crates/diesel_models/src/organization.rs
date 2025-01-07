use common_utils::{id_type, pii};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};

#[cfg(feature = "v1")]
use crate::schema::organization;
#[cfg(feature = "v2")]
use crate::schema_v2::organization;
pub trait OrganizationBridge {
    fn get_organization_id(&self) -> id_type::OrganizationId;
    fn get_organization_name(&self) -> Option<String>;
    fn set_organization_name(&mut self, organization_name: String);
    fn set_platform_merchant_id(&mut self, platform_merchant_id: id_type::MerchantId);
    fn get_platform_merchant_id(&self) -> Option<id_type::MerchantId>;
}
#[cfg(feature = "v1")]
#[derive(Clone, Debug, Identifiable, Queryable, Selectable)]
#[diesel(
    table_name = organization,
    primary_key(org_id),
    check_for_backend(diesel::pg::Pg)
)]
pub struct Organization {
    org_id: id_type::OrganizationId,
    org_name: Option<String>,
    pub organization_details: Option<pii::SecretSerdeValue>,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub created_at: time::PrimitiveDateTime,
    pub modified_at: time::PrimitiveDateTime,
    #[allow(dead_code)]
    id: Option<id_type::OrganizationId>,
    #[allow(dead_code)]
    organization_name: Option<String>,
    pub platform_merchant_id: Option<id_type::MerchantId>,
}

#[cfg(feature = "v2")]
#[derive(Clone, Debug, Identifiable, Queryable, Selectable)]
#[diesel(
    table_name = organization,
    primary_key(id),
    check_for_backend(diesel::pg::Pg)
)]
pub struct Organization {
    pub organization_details: Option<pii::SecretSerdeValue>,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub created_at: time::PrimitiveDateTime,
    pub modified_at: time::PrimitiveDateTime,
    id: id_type::OrganizationId,
    organization_name: Option<String>,
    pub platform_merchant_id: Option<id_type::MerchantId>,
}

#[cfg(feature = "v1")]
impl Organization {
    pub fn new(org_new: OrganizationNew) -> Self {
        let OrganizationNew {
            org_id,
            org_name,
            organization_details,
            metadata,
            created_at,
            modified_at,
            id: _,
            organization_name: _,
            platform_merchant_id,
        } = org_new;
        Self {
            id: Some(org_id.clone()),
            organization_name: org_name.clone(),
            org_id,
            org_name,
            organization_details,
            metadata,
            created_at,
            modified_at,
            platform_merchant_id,
        }
    }
}

#[cfg(feature = "v2")]
impl Organization {
    pub fn new(org_new: OrganizationNew) -> Self {
        let OrganizationNew {
            id,
            organization_name,
            organization_details,
            metadata,
            created_at,
            modified_at,
            platform_merchant_id,
        } = org_new;
        Self {
            id,
            organization_name,
            organization_details,
            metadata,
            created_at,
            modified_at,
            platform_merchant_id,
        }
    }
}

#[cfg(feature = "v1")]
#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = organization, primary_key(org_id))]
pub struct OrganizationNew {
    org_id: id_type::OrganizationId,
    org_name: Option<String>,
    id: Option<id_type::OrganizationId>,
    organization_name: Option<String>,
    pub organization_details: Option<pii::SecretSerdeValue>,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub created_at: time::PrimitiveDateTime,
    pub modified_at: time::PrimitiveDateTime,
    pub platform_merchant_id: Option<id_type::MerchantId>,
}

#[cfg(feature = "v2")]
#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = organization, primary_key(id))]
pub struct OrganizationNew {
    id: id_type::OrganizationId,
    organization_name: Option<String>,
    pub organization_details: Option<pii::SecretSerdeValue>,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub created_at: time::PrimitiveDateTime,
    pub modified_at: time::PrimitiveDateTime,
    pub platform_merchant_id: Option<id_type::MerchantId>,
}

#[cfg(feature = "v1")]
impl OrganizationNew {
    pub fn new(id: id_type::OrganizationId, organization_name: Option<String>) -> Self {
        Self {
            org_id: id.clone(),
            org_name: organization_name.clone(),
            id: Some(id),
            organization_name,
            organization_details: None,
            metadata: None,
            created_at: common_utils::date_time::now(),
            modified_at: common_utils::date_time::now(),
            platform_merchant_id: None,
        }
    }
}

#[cfg(feature = "v2")]
impl OrganizationNew {
    pub fn new(id: id_type::OrganizationId, organization_name: Option<String>) -> Self {
        Self {
            id,
            organization_name,
            organization_details: None,
            metadata: None,
            created_at: common_utils::date_time::now(),
            modified_at: common_utils::date_time::now(),
            platform_merchant_id: None,
        }
    }
}

#[cfg(feature = "v1")]
#[derive(Clone, Debug, AsChangeset)]
#[diesel(table_name = organization)]
pub struct OrganizationUpdateInternal {
    org_name: Option<String>,
    organization_name: Option<String>,
    organization_details: Option<pii::SecretSerdeValue>,
    metadata: Option<pii::SecretSerdeValue>,
    modified_at: time::PrimitiveDateTime,
    platform_merchant_id: Option<id_type::MerchantId>,
}

#[cfg(feature = "v2")]
#[derive(Clone, Debug, AsChangeset)]
#[diesel(table_name = organization)]
pub struct OrganizationUpdateInternal {
    organization_name: Option<String>,
    organization_details: Option<pii::SecretSerdeValue>,
    metadata: Option<pii::SecretSerdeValue>,
    modified_at: time::PrimitiveDateTime,
    platform_merchant_id: Option<id_type::MerchantId>,
}

pub enum OrganizationUpdate {
    Update {
        organization_name: Option<String>,
        organization_details: Option<pii::SecretSerdeValue>,
        metadata: Option<pii::SecretSerdeValue>,
    },
    ToPlatformAccount {
        platform_merchant_id: id_type::MerchantId,
    },
}

#[cfg(feature = "v1")]
impl From<OrganizationUpdate> for OrganizationUpdateInternal {
    fn from(value: OrganizationUpdate) -> Self {
        match value {
            OrganizationUpdate::Update {
                organization_name,
                organization_details,
                metadata,
            } => Self {
                org_name: organization_name.clone(),
                organization_name,
                organization_details,
                metadata,
                modified_at: common_utils::date_time::now(),
                platform_merchant_id: None,
            },
            OrganizationUpdate::ToPlatformAccount {
                platform_merchant_id,
            } => Self {
                org_name: None,
                organization_name: None,
                organization_details: None,
                metadata: None,
                modified_at: common_utils::date_time::now(),
                platform_merchant_id: Some(platform_merchant_id),
            },
        }
    }
}

#[cfg(feature = "v2")]
impl From<OrganizationUpdate> for OrganizationUpdateInternal {
    fn from(value: OrganizationUpdate) -> Self {
        match value {
            OrganizationUpdate::Update {
                organization_name,
                organization_details,
                metadata,
            } => Self {
                organization_name,
                organization_details,
                metadata,
                modified_at: common_utils::date_time::now(),
                platform_merchant_id: None,
            },
            OrganizationUpdate::ToPlatformAccount {
                platform_merchant_id,
            } => Self {
                organization_name: None,
                organization_details: None,
                metadata: None,
                modified_at: common_utils::date_time::now(),
                platform_merchant_id: Some(platform_merchant_id),
            },
        }
    }
}

#[cfg(feature = "v1")]
impl OrganizationBridge for Organization {
    fn get_organization_id(&self) -> id_type::OrganizationId {
        self.org_id.clone()
    }
    fn get_organization_name(&self) -> Option<String> {
        self.org_name.clone()
    }
    fn set_organization_name(&mut self, organization_name: String) {
        self.org_name = Some(organization_name);
    }
    fn set_platform_merchant_id(&mut self, platform_merchant_id: id_type::MerchantId) {
        self.platform_merchant_id = Some(platform_merchant_id);
    }
    fn get_platform_merchant_id(&self) -> Option<id_type::MerchantId> {
        self.platform_merchant_id.clone()
    }
}

#[cfg(feature = "v1")]
impl OrganizationBridge for OrganizationNew {
    fn get_organization_id(&self) -> id_type::OrganizationId {
        self.org_id.clone()
    }
    fn get_organization_name(&self) -> Option<String> {
        self.org_name.clone()
    }
    fn set_organization_name(&mut self, organization_name: String) {
        self.org_name = Some(organization_name);
    }
    fn set_platform_merchant_id(&mut self, platform_merchant_id: id_type::MerchantId) {
        self.platform_merchant_id = Some(platform_merchant_id);
    }
    fn get_platform_merchant_id(&self) -> Option<id_type::MerchantId> {
        self.platform_merchant_id.clone()
    }
}

#[cfg(feature = "v2")]
impl OrganizationBridge for Organization {
    fn get_organization_id(&self) -> id_type::OrganizationId {
        self.id.clone()
    }
    fn get_organization_name(&self) -> Option<String> {
        self.organization_name.clone()
    }
    fn set_organization_name(&mut self, organization_name: String) {
        self.organization_name = Some(organization_name);
    }
    fn set_platform_merchant_id(&mut self, platform_merchant_id: id_type::MerchantId) {
        self.platform_merchant_id = Some(platform_merchant_id);
    }
    fn get_platform_merchant_id(&self) -> Option<id_type::MerchantId> {
        self.platform_merchant_id.clone()
    }
}

#[cfg(feature = "v2")]
impl OrganizationBridge for OrganizationNew {
    fn get_organization_id(&self) -> id_type::OrganizationId {
        self.id.clone()
    }
    fn get_organization_name(&self) -> Option<String> {
        self.organization_name.clone()
    }
    fn set_organization_name(&mut self, organization_name: String) {
        self.organization_name = Some(organization_name);
    }
    fn set_platform_merchant_id(&mut self, platform_merchant_id: id_type::MerchantId) {
        self.platform_merchant_id = Some(platform_merchant_id);
    }
    fn get_platform_merchant_id(&self) -> Option<id_type::MerchantId> {
        self.platform_merchant_id.clone()
    }
}
