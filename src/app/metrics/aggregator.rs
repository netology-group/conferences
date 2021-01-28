use std::str::FromStr;
use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use svc_agent::AgentId;

use crate::app::context::GlobalContext;
use crate::app::metrics::{Metric, MetricKey, PercentileReport, Tags};

pub(crate) struct Aggregator<'a, C: GlobalContext> {
    context: &'a C,
}

impl<'a, C: GlobalContext> Aggregator<'a, C> {
    pub(crate) fn new(context: &'a C) -> Self {
        Self { context }
    }

    pub(crate) fn get(&self) -> anyhow::Result<Vec<crate::app::metrics::Metric>> {
        let now = Utc::now();
        let mut metrics = vec![];

        append_mqtt_stats(&mut metrics, self.context, now)?;
        append_internal_stats(&mut metrics, self.context, now);
        append_redis_pool_metrics(&mut metrics, self.context, now);
        append_dynamic_stats(&mut metrics, self.context, now)?;

        append_janus_stats(&mut metrics, self.context, now)?;

        if let Some(counter) = self.context.running_requests() {
            let tags = Tags::build_internal_tags(crate::APP_VERSION, &self.context.agent_id());
            metrics.push(Metric::new(
                MetricKey::RunningRequests,
                counter.load(Ordering::SeqCst),
                now,
                tags,
            ));
        }

        Ok(metrics)
    }
}

fn append_mqtt_stats(
    metrics: &mut Vec<Metric>,
    context: &impl GlobalContext,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    if let Some(qc) = context.queue_counter() {
        let stats = qc
            .get_stats()
            .map_err(|err| anyhow!(err).context("Failed to get stats"))?;

        stats.into_iter().for_each(|(tags, value)| {
            let tags = Tags::build_queues_tags(crate::APP_VERSION, context.agent_id(), tags);

            if value.incoming_requests > 0 {
                metrics.push(Metric::new(
                    MetricKey::IncomingQueueRequests,
                    value.incoming_requests,
                    now,
                    tags.clone(),
                ));
            }
            if value.incoming_responses > 0 {
                metrics.push(Metric::new(
                    MetricKey::IncomingQueueResponses,
                    value.incoming_responses,
                    now,
                    tags.clone(),
                ));
            }
            if value.incoming_events > 0 {
                metrics.push(Metric::new(
                    MetricKey::IncomingQueueEvents,
                    value.incoming_events,
                    now,
                    tags.clone(),
                ));
            }
            if value.outgoing_requests > 0 {
                metrics.push(Metric::new(
                    MetricKey::OutgoingQueueRequests,
                    value.outgoing_requests,
                    now,
                    tags.clone(),
                ));
            }
            if value.outgoing_responses > 0 {
                metrics.push(Metric::new(
                    MetricKey::OutgoingQueueResponses,
                    value.outgoing_responses,
                    now,
                    tags.clone(),
                ));
            }
            if value.outgoing_events > 0 {
                metrics.push(Metric::new(
                    MetricKey::OutgoingQueueEvents,
                    value.outgoing_events,
                    now,
                    tags,
                ));
            }
        });
    }

    Ok(())
}

fn append_internal_stats(
    metrics: &mut Vec<Metric>,
    context: &impl GlobalContext,
    now: DateTime<Utc>,
) {
    let tags = Tags::build_internal_tags(crate::APP_VERSION, context.agent_id());
    let state = context.db().state();

    metrics.extend_from_slice(&[
        Metric::new(
            MetricKey::DbConnections,
            state.connections,
            now,
            tags.clone(),
        ),
        Metric::new(
            MetricKey::IdleDbConnections,
            state.idle_connections,
            now,
            tags,
        ),
    ])
}

fn append_redis_pool_metrics(
    metrics: &mut Vec<Metric>,
    context: &impl GlobalContext,
    now: DateTime<Utc>,
) {
    if let Some(pool) = context.redis_pool() {
        let state = pool.state();
        let tags = Tags::build_internal_tags(crate::APP_VERSION, context.agent_id());

        metrics.extend_from_slice(&[
            Metric::new(
                MetricKey::RedisConnections,
                state.connections,
                now,
                tags.clone(),
            ),
            Metric::new(
                MetricKey::IdleRedisConnections,
                state.idle_connections,
                now,
                tags,
            ),
        ]);
    }
}

fn append_dynamic_stats(
    metrics: &mut Vec<Metric>,
    context: &impl GlobalContext,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    if let Some(dynamic_stats) = context.dynamic_stats() {
        for (key, value) in dynamic_stats.flush()? {
            metrics.push(Metric::new(
                MetricKey::Dynamic(key),
                value,
                now,
                Tags::Empty,
            ));
        }

        for (agent_id, value) in dynamic_stats.get_janus_timeouts()? {
            let tags = Tags::build_janus_tags(
                crate::APP_VERSION,
                context.agent_id(),
                &AgentId::from_str(&agent_id)
                    .expect("We only write agent ids into DynamicStatsCollector"),
            );

            metrics.push(Metric::new(
                MetricKey::JanusTimeoutsTotal,
                value,
                now,
                tags.clone(),
            ));
        }

        for (method, PercentileReport { p95, p99, max }) in dynamic_stats.get_handler_timings()? {
            let tags =
                Tags::build_running_futures_tags(crate::APP_VERSION, context.agent_id(), method);

            metrics.push(Metric::new(
                MetricKey::RunningRequestDurationP95,
                p95,
                now,
                tags.clone(),
            ));

            metrics.push(Metric::new(
                MetricKey::RunningRequestDurationP99,
                p99,
                now,
                tags.clone(),
            ));

            metrics.push(Metric::new(
                MetricKey::RunningRequestDurationMax,
                max,
                now,
                tags.clone(),
            ));
        }
    }

    Ok(())
}

fn append_janus_stats(
    metrics: &mut Vec<Metric>,
    context: &impl GlobalContext,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    use crate::db::agent;
    use anyhow::Context;

    match context.get_conn() {
        Err(e) => {
            error!(
                crate::LOG,
                "Aggregator failed to acquire connection, reason = {:?}", e
            );
            Ok(())
        }
        Ok(conn) => {
            let tags = Tags::build_internal_tags(crate::APP_VERSION, context.agent_id());

            // The number of online janus backends.
            let online_backends_count = crate::db::janus_backend::count(&conn)
                .context("Failed to get janus backends count")?;

            metrics.push(Metric::new(
                MetricKey::OnlineJanusBackendsCount,
                online_backends_count,
                now,
                tags.clone(),
            ));

            // Total capacity of online janus backends.
            let total_capacity = crate::db::janus_backend::total_capacity(&conn)
                .context("Failed to get janus backends total capacity")?;

            metrics.push(Metric::new(
                MetricKey::JanusBackendTotalCapacity,
                total_capacity,
                now,
                tags.clone(),
            ));

            // The number of agents connect to an RTC.
            let connected_agents_count = agent::ConnectedCountQuery::new()
                .execute(&conn)
                .context("Failed to get connected agents count")?;

            metrics.push(Metric::new(
                MetricKey::ConnectedAgentsCount,
                connected_agents_count,
                now,
                tags,
            ));

            let backend_load = crate::db::janus_backend::reserve_load_for_each_backend(&conn)
                .context("Failed to get janus backends reserve load")?
                .into_iter()
                .fold(vec![], |mut v, load_row| {
                    let tags = Tags::build_janus_tags(
                        crate::APP_VERSION,
                        context.agent_id(),
                        &load_row.backend_id,
                    );

                    v.push(Metric::new(
                        MetricKey::JanusBackendReserveLoad,
                        load_row.load,
                        now,
                        tags.clone(),
                    ));
                    v.push(Metric::new(
                        MetricKey::JanusBackendAgentLoad,
                        load_row.taken,
                        now,
                        tags,
                    ));
                    v
                });

            metrics.extend(backend_load);

            Ok(())
        }
    }
}