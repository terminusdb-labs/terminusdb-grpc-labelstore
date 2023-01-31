include!(concat!(env!("OUT_DIR"), concat!("/labels.rs")));

impl LayerId {
    pub fn new(id: [u32; 5]) -> Self {
        Self {
            v1: id[0],
            v2: id[1],
            v3: id[2],
            v4: id[3],
            v5: id[4],
        }
    }
    pub fn id(&self) -> [u32; 5] {
        [self.v1, self.v2, self.v3, self.v4, self.v5]
    }
}

impl GetLabelResponse {
    pub fn new(id: Option<[u32; 5]>, version: u64) -> Self {
        let layer = id.map(|id| LayerId {
            v1: id[0],
            v2: id[1],
            v3: id[2],
            v4: id[3],
            v5: id[4],
        });

        Self { layer, version }
    }

    pub fn id(&self) -> Option<[u32; 5]> {
        self.layer.as_ref().map(|l| l.id())
    }
}

impl Label {
    pub fn new<S: Into<prost::alloc::string::String>>(
        name: S,
        layer: Option<[u32; 5]>,
        version: u64,
    ) -> Self {
        Self {
            name: name.into(),
            layer: layer.map(LayerId::new),
            version,
        }
    }
}
