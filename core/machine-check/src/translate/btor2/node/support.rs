use btor2rs::{Bitvec, Nid, Sid, Sort};

use super::StmtTranslator;
use anyhow::anyhow;

impl<'a> StmtTranslator<'a> {
    pub(super) fn get_sort(&self, sid: Sid) -> Result<&Sort, anyhow::Error> {
        self.translator
            .btor2
            .sorts
            .get(&sid)
            .ok_or_else(|| anyhow!("Unknown sort"))
    }

    pub(super) fn get_bitvec(&self, sid: Sid) -> Result<&Bitvec, anyhow::Error> {
        let sort = self.get_sort(sid)?;
        let Sort::Bitvec(bitvec) = sort else {
        return Err(anyhow!("Expected bitvec sort"));
    };
        Ok(bitvec)
    }

    pub(super) fn get_nid_bitvec(&self, nid: Nid) -> Result<&Bitvec, anyhow::Error> {
        let node = self
            .translator
            .btor2
            .nodes
            .get(&nid)
            .ok_or_else(|| anyhow!("Unknown node"))?;
        let sid = node
            .get_sid()
            .ok_or_else(|| anyhow!("Expected node with sid"))?;
        self.get_bitvec(sid)
    }
}
