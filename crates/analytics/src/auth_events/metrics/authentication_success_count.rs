use std::collections::HashSet;

use api_models::analytics::{
    auth_events::AuthEventMetricsBucketIdentifier, Granularity, TimeRange,
};
use common_utils::errors::ReportSwitchExt;
use error_stack::ResultExt;
use time::PrimitiveDateTime;

use super::AuthEventMetricRow;
use crate::{
    query::{Aggregate, GroupByClause, QueryBuilder, QueryFilter, ToSql, Window},
    types::{AnalyticsCollection, AnalyticsDataSource, MetricsError, MetricsResult},
};

#[derive(Default)]
pub(super) struct AuthenticationSuccessCount;

#[async_trait::async_trait]
impl<T> super::AuthEventMetric<T> for AuthenticationSuccessCount
where
    T: AnalyticsDataSource + super::AuthEventMetricAnalytics,
    PrimitiveDateTime: ToSql<T>,
    AnalyticsCollection: ToSql<T>,
    Granularity: GroupByClause<T>,
    Aggregate<&'static str>: ToSql<T>,
    Window<&'static str>: ToSql<T>,
{
    async fn load_metrics(
        &self,
        _merchant_id: &common_utils::id_type::MerchantId,

        granularity: &Option<Granularity>,
        time_range: &TimeRange,
        pool: &T,
    ) -> MetricsResult<HashSet<(AuthEventMetricsBucketIdentifier, AuthEventMetricRow)>> {
        let mut query_builder: QueryBuilder<T> =
            QueryBuilder::new(AnalyticsCollection::Authentications);

        query_builder
            .add_select_column(Aggregate::Count {
                field: Some("authentication_id"),
                alias: Some("count"),
            })
            .switch()?;

        if let Some(granularity) = granularity.as_ref() {
            query_builder
                .add_granularity_in_mins(granularity)
                .switch()?;
        }

        query_builder
            .add_filter_clause("merchant_id", _merchant_id)
            .switch()?;

        query_builder
            .add_filter_clause("authentication_status", "success")
            .switch()?;

        // query_builder
        //     .add_bool_filter_clause("first_event", 1)
        //     .switch()?;

        // query_builder
        //     .add_filter_clause("event_name", SdkEventNames::AuthenticationCall)
        //     .switch()?;

        // query_builder
        //     .add_filter_clause("log_type", "INFO")
        //     .switch()?;

        // query_builder
        //     .add_filter_clause("category", "API")
        //     .switch()?;

        time_range
            .set_filter_clause(&mut query_builder)
            .attach_printable("Error filtering time range")
            .switch()?;

        if let Some(_granularity) = granularity.as_ref() {
            query_builder
                .add_group_by_clause("time_bucket")
                .attach_printable("Error adding granularity")
                .switch()?;
        }

        query_builder
            .execute_query::<AuthEventMetricRow, _>(pool)
            .await
            .change_context(MetricsError::QueryBuildingError)?
            .change_context(MetricsError::QueryExecutionFailure)?
            .into_iter()
            .map(|i| {
                Ok((
                    AuthEventMetricsBucketIdentifier::new(i.time_bucket.clone()),
                    i,
                ))
            })
            .collect::<error_stack::Result<
                HashSet<(AuthEventMetricsBucketIdentifier, AuthEventMetricRow)>,
                crate::query::PostProcessingError,
            >>()
            .change_context(MetricsError::PostProcessingFailure)
    }
}
