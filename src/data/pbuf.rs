use std::io::Write;

use data::pod;
use protobuf::Message;

use byteorder::{WriteBytesExt, LittleEndian};

use Dx16Result;

#[path="./protobuf/dazone.rs"]
mod pbufdata;

pub use self::pbufdata::{Ranking, UserVisits};

pub trait Protobufable {
    fn write_to_pbuf<W:Write>(&self, w:&mut W) -> Dx16Result<()>;
}

impl Protobufable for pod::Ranking {
    fn write_to_pbuf<W:Write>(&self, w:&mut W) -> Dx16Result<()> {
        let mut ranking = pbufdata::Ranking::default();
        ranking.set_url(self.url.clone());
        ranking.set_pagerank(self.pagerank);
        ranking.set_duration(self.duration);
        let bytes = ranking.write_to_bytes().unwrap();
        w.write_u16::<LittleEndian>(bytes.len() as u16).unwrap();
        w.write_all(&*bytes).unwrap();
        Ok(())
    }
}

impl Protobufable for pod::UserVisits {
    fn write_to_pbuf<W:Write>(&self, w:&mut W) -> Dx16Result<()> {
        let mut it = pbufdata::UserVisits::default();
        it.set_sourceIP(self.source_ip.clone());
        it.set_destURL(self.dest_url.clone());
        it.set_visitDate(self.visit_date.clone());
        it.set_adRevenue(self.ad_revenue);
        it.set_userAgent(self.user_agent.clone());
        it.set_countryCode(self.country_code.clone());
        it.set_languageCode(self.language_code.clone());
        it.set_searchWord(self.search_word.clone());
        it.set_duration(self.duration as u64);
        let bytes = it.write_to_bytes().unwrap();
        w.write_u16::<LittleEndian>(bytes.len() as u16).unwrap();
        w.write_all(&*bytes).unwrap();
        Ok(())
    }
}
