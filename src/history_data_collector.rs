use std::{collections::HashMap, time::Instant};

use uuid::Uuid;

use crate::{gizmos::GizmoResult, temperature::Temp};

pub struct DataPoint {
    pub temperature: Temp,
    pub time: Instant,
}

pub struct HistoryDataCollector {
    //
    // Stores data points for each gizmo UUID
    //
    pub stored_data: HashMap<Uuid, Vec<DataPoint>>,
}

impl HistoryDataCollector {
    pub fn new() -> Self {
        Self {
            stored_data: HashMap::new(),
        }
    }

    pub fn add_from_gizmo_results(
        &mut self,
        time: Instant,
        gizmo_results: &HashMap<Uuid, GizmoResult>,
    ) -> Result<(), anyhow::Error> {
        for (gizmo_uuid, gizmo_result) in gizmo_results {
            let data_point = DataPoint {
                temperature: gizmo_result.temperature,
                time,
            };

            let data_points = self.stored_data.entry(*gizmo_uuid).or_default();

            data_points.push(data_point);
        }
        Ok(())
    }

    pub fn for_each_data_point<F>(&self, gizmo_uuid: Uuid, from: Instant, to: Instant, mut f: F)
    where
        F: FnMut(&DataPoint),
    {
        // TODO: binary search
        self.stored_data.get(&gizmo_uuid).inspect(|data_points| {
            for data_point in data_points.iter() {
                if data_point.time >= from && data_point.time <= to {
                    f(data_point);
                }
            }
        });
    }
}
