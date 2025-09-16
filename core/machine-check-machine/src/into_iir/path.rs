use machine_check_common::iir::path::{IPath, IPathSegment};

use crate::wir::WPath;

impl WPath {
    pub fn into_iir(self) -> IPath {
        IPath {
            leading_colon: self.leading_colon.map(|span| span.first()),
            segments: self
                .segments
                .into_iter()
                .map(|segment| IPathSegment {
                    ident: segment.ident.into_iir(),
                })
                .collect(),
        }
    }
}
