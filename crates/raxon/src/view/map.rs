//! Native map view (MKMapView on iOS).

use crate::dom::{Attribute, Tree, WidgetId};

use super::view::View;

/// A native map view centered at a coordinate with an optional visible span
/// and zero or more annotations.
pub struct MapView {
    latitude: f64,
    longitude: f64,
    lat_span: f64,
    lon_span: f64,
    annotations: Vec<(String, f64, f64, String)>, // (id, lat, lon, title)
}

/// Create a map view centered at the given coordinates.
///
/// ```rust,ignore
/// map_view(37.7749, -122.4194)
///     .span(0.05, 0.05)
///     .annotation("hq", 37.7749, -122.4194, "SF HQ")
///     .grow()
/// ```
pub fn map_view(latitude: f64, longitude: f64) -> MapView {
    MapView {
        latitude,
        longitude,
        lat_span: 0.01,
        lon_span: 0.01,
        annotations: vec![],
    }
}

impl MapView {
    /// Set the visible region span in degrees (smaller = more zoomed in).
    pub fn span(mut self, lat_span: f64, lon_span: f64) -> Self {
        self.lat_span = lat_span;
        self.lon_span = lon_span;
        self
    }

    /// Add an annotation pin at the given coordinate.
    ///
    /// `id` uniquely identifies this annotation for future updates or removal.
    pub fn annotation(
        mut self,
        id: impl Into<String>,
        lat: f64,
        lon: f64,
        title: impl Into<String>,
    ) -> Self {
        self.annotations.push((id.into(), lat, lon, title.into()));
        self
    }
}

impl View for MapView {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_map_view();
        tree.set(id, Attribute::MapCenter { latitude: self.latitude, longitude: self.longitude });
        tree.set(id, Attribute::MapSpan { lat_span: self.lat_span, lon_span: self.lon_span });
        for (ann_id, lat, lon, title) in self.annotations {
            tree.set(
                id,
                Attribute::MapAnnotation {
                    annotation_id: ann_id,
                    latitude: lat,
                    longitude: lon,
                    title,
                },
            );
        }
        id
    }
}
