use machine_check_common::iir::path::{IPath, IPathSegment};

use crate::wir::WTotalPath;

impl WTotalPath {
    pub fn into_iir(self) -> IPath {
        IPath {
            leading_colon: self.leading_colon.map(|span| span.into_iir()),
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
