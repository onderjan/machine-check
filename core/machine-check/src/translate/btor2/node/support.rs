use btor2rs::{Bitvec, Nid, Sid, Sort};

use crate::translate::btor2::Error;

use super::NodeTranslator;

impl<'a> NodeTranslator<'a> {
    pub(super) fn get_sort(&self, sid: Sid) -> Result<&Sort, Error> {
        self.translator
            .btor2
            .sorts
            .get(&sid)
            .ok_or(Error::InvalidSort(sid))
    }

    pub(super) fn get_bitvec(&self, sid: Sid) -> Result<&Bitvec, Error> {
        let sort = self.get_sort(sid)?;
        let Sort::Bitvec(bitvec) = sort else {
        return Err(Error::ExpectBitvecSort(sid));
    };
        Ok(bitvec)
    }

    pub(super) fn get_nid_bitvec(&self, nid: Nid) -> Result<&Bitvec, Error> {
        let node = self
            .translator
            .btor2
            .nodes
            .get(&nid)
            .ok_or(Error::InvalidNode(nid))?;
        let sid = node.get_sid().ok_or(Error::UnknownNodeSort(nid))?;
        self.get_bitvec(sid)
    }
}
