use std::sync::Arc;
use tracing::{debug, error, info};

use crate::collector::calendar::CalendarCollector;
use crate::ws::{self, Hub};

pub struct CalendarPipeline {
    collector: Arc<CalendarCollector>,
    hub: Arc<Hub>,
}

impl CalendarPipeline {
    pub fn new(collector: Arc<CalendarCollector>, hub: Arc<Hub>) -> Self {
        Self { collector, hub }
    }

    pub async fn run(&self) {
        debug!("calendar pipeline: checking");

        let events: Vec<crate::collector::calendar::CalendarEvent> = match self.collector.get_upcoming_high_impact(15, 5).await {
            Ok(e) => e,
            Err(e) => {
                error!(error = %e, "calendar pipeline: failed");
                return;
            }
        };

        if events.is_empty() {
            debug!("calendar pipeline: no upcoming events");
            return;
        }

        let mut broadcasted = 0u32;
        for event in &events {
            let event_data = serde_json::json!({
                "event_id": event.event_id,
                "title": event.title,
                "country": event.country,
                "currency": event.currency,
                "date_wib": event.date_wib,
                "impact": event.impact,
                "forecast": event.forecast,
                "previous": event.previous,
                "minutes_until": event.minutes_until,
            });

            let data = serde_json::json!({ "calendar_event": event_data });
            let count = self.hub.broadcast(ws::EVENT_CALENDAR_REMINDER, data, "calendar").await;

            info!(
                event = %event.title,
                clients = count,
                minutes_until = event.minutes_until,
                "calendar broadcast ok"
            );
            broadcasted += 1;
        }

        info!(
            events_found = events.len(),
            broadcasted,
            "calendar pipeline: completed"
        );
    }
}
